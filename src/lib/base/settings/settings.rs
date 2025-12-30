use std::sync::OnceLock;

use crate::base::fs::FileReader;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::paths::Paths;
use crate::base::format::Sensitive;
use crate::base::http::{Authorization, HttpClient, RequestConfig};
use crate::jira::types::JiraUser;
use crate::pr::GitHub;
use std::collections::HashMap;

// ==================== 返回结构体 ====================

/// 日志配置信息
#[derive(Debug, Clone)]
pub struct LogConfigInfo {
    /// 日志输出文件夹名称
    pub output_folder_name: String,
    /// 日志下载基础目录
    pub download_base_dir: Option<String>,
}

/// LLM 配置信息
#[derive(Debug, Clone)]
pub struct LLMConfigInfo {
    /// Provider
    pub provider: String,
    /// Model（包含 URL 信息，如果适用）
    pub model: String,
    /// Key（掩码显示）
    pub key: String,
    /// Output Language
    pub language: String,
}

/// Jira 验证结果
#[derive(Debug, Clone)]
pub struct JiraVerificationResult {
    /// 是否已配置
    pub configured: bool,
    /// 配置信息（如果已配置）
    pub config: Option<JiraConfigInfo>,
    /// 验证结果
    pub verification: Option<JiraVerificationStatus>,
}

/// Jira 配置信息
#[derive(Debug, Clone)]
pub struct JiraConfigInfo {
    /// 邮箱
    pub email: String,
    /// 服务地址
    pub service_address: String,
    /// API Token（掩码显示）
    pub api_token: String,
}

/// Jira 验证状态
#[derive(Debug, Clone)]
pub enum JiraVerificationStatus {
    /// 验证成功
    Success { email: String, account_id: String },
    /// 验证失败
    Failed {
        reason: String,
        details: Vec<String>,
    },
}

/// GitHub 验证结果
#[derive(Debug, Clone)]
pub struct GitHubVerificationResult {
    /// 是否已配置
    pub configured: bool,
    /// 账号列表
    pub accounts: Vec<GitHubAccountInfo>,
    /// 验证总结
    pub summary: GitHubVerificationSummary,
}

/// GitHub 账号信息
#[derive(Debug, Clone)]
pub struct GitHubAccountInfo {
    /// 账号名称
    pub name: String,
    /// 是否当前账号
    pub is_current: bool,
    /// 邮箱
    pub email: String,
    /// API Token（掩码显示）
    pub token: String,
    /// 验证状态
    pub verification_status: String,
    /// 验证错误信息（如果验证失败）
    pub verification_error: Option<String>,
}

/// GitHub 验证总结
#[derive(Debug, Clone)]
pub struct GitHubVerificationSummary {
    /// 总账号数
    pub total_count: usize,
    /// 成功数
    pub success_count: usize,
    /// 失败账号列表
    pub failed_accounts: Vec<String>,
}

/// 配置验证结果
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// 日志配置
    pub log: LogConfigInfo,
    /// LLM 配置
    pub llm: LLMConfigInfo,
    /// Jira 验证结果
    pub jira: JiraVerificationResult,
    /// GitHub 验证结果
    pub github: GitHubVerificationResult,
}

// ==================== TOML 配置结构体 ====================

/// Jira 配置（TOML）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JiraSettings {
    /// Jira 用户邮箱（用于 API 认证）
    pub email: Option<String>,
    /// Jira API Token
    pub api_token: Option<String>,
    /// Jira 服务地址
    pub service_address: Option<String>,
}

impl JiraSettings {
    /// 检查 JIRA 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.email.is_none() && self.api_token.is_none() && self.service_address.is_none()
    }
}

/// GitHub 账号配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAccount {
    /// 账号名称（用于标识和切换）
    pub name: String,
    /// 账号邮箱（必填，用于显示和区分）
    pub email: String,
    /// GitHub API Token
    pub api_token: String,
}

/// GitHub 配置（TOML）
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitHubSettings {
    /// 多个 GitHub 账号列表
    #[serde(default)]
    pub accounts: Vec<GitHubAccount>,
    /// 当前激活的账号名称
    pub current: Option<String>,
}

impl GitHubSettings {
    /// 检查 GitHub 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty() && self.current.is_none()
    }

    /// 获取当前激活的账号
    ///
    /// 如果设置了 `current`，返回对应的账号；否则返回第一个账号。
    /// 如果没有账号，返回 `None`。
    pub fn get_current_account(&self) -> Option<&GitHubAccount> {
        if self.accounts.is_empty() {
            return None;
        }

        if let Some(ref current_name) = self.current {
            self.accounts.iter().find(|acc| acc.name == *current_name)
        } else {
            // 如果没有设置 current，返回第一个账号
            self.accounts.first()
        }
    }

    /// 获取当前账号的 API Token
    pub fn get_current_token(&self) -> Option<&str> {
        self.get_current_account().map(|acc| acc.api_token.as_str())
    }
}

/// 默认下载基础目录路径
///
/// 跨平台支持：
/// - Unix (macOS/Linux): `~/Documents/Workflow`
/// - Windows: `%USERPROFILE%\Documents\Workflow`
pub fn default_download_base_dir() -> String {
    // 使用 dirs::home_dir() 获取主目录
    dirs::home_dir()
        .map(|h| h.join("Documents").join("Workflow").to_string_lossy().to_string())
        .unwrap_or_else(|| {
            if cfg!(target_os = "windows") {
                "C:\\Users\\User\\Documents\\Workflow".to_string()
            } else {
                "~/Documents/Workflow".to_string()
            }
        })
}

/// 日志配置（TOML）
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSettings {
    /// 日志输出文件夹名称
    /// 如果为 `None`，使用默认值 `logs`，且不写入配置文件
    #[serde(default = "LogSettings::default_log_folder_option")]
    pub output_folder_name: Option<String>,
    /// 日志下载基础目录
    #[serde(default = "LogSettings::default_download_base_dir_option")]
    pub download_base_dir: Option<String>,
    /// 日志级别（none, error, warn, info, debug）
    pub level: Option<String>,
    /// 是否同时输出 tracing 日志到控制台（stderr）
    /// 如果为 `true`，tracing 日志会同时输出到文件和控制台
    /// 如果配置文件中不存在此字段，默认为 `false`（只输出到文件）
    /// 注意：只有设置为 `true` 时才会写入配置文件，设置为 `false` 时从配置文件中删除
    pub enable_trace_console: Option<bool>,
}

impl LogSettings {
    /// 检查日志配置是否为空（所有字段都是默认值）
    pub fn is_empty(&self) -> bool {
        let default = LogSettings::default();
        self.output_folder_name == default.output_folder_name
            && self.download_base_dir == default.download_base_dir
            && self.level == default.level
            && self.enable_trace_console == default.enable_trace_console
    }

    /// 默认日志文件夹名称
    pub fn default_log_folder() -> String {
        "logs".to_string()
    }

    /// 默认日志文件夹名称（Option 类型，用于序列化）
    pub fn default_log_folder_option() -> Option<String> {
        None // None 表示使用默认值，不写入配置文件
    }

    /// 获取日志文件夹名称（如果为 None，返回默认值）
    pub fn get_output_folder_name(&self) -> String {
        self.output_folder_name.clone().unwrap_or_else(Self::default_log_folder)
    }

    /// 默认下载基础目录路径（Option 类型，用于序列化）
    /// 返回 `None` 表示使用默认值，不写入配置文件
    pub fn default_download_base_dir_option() -> Option<String> {
        None // None 表示使用默认值，不写入配置文件
    }
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            output_folder_name: Self::default_log_folder_option(), // None
            download_base_dir: Self::default_download_base_dir_option(), // None
            level: None,
            enable_trace_console: None,
        }
    }
}
//         self.project_id.is_none() && self.csrf_token.is_none() && self.cookie.is_none()
//     }
// }

// ==================== TOML LLM 配置结构体 ====================

/// 单个 LLM Provider 的配置
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LLMProviderSettings {
    /// Provider URL（仅 proxy 使用）
    pub url: Option<String>,
    /// Provider API Key
    pub key: Option<String>,
    /// 模型名称
    pub model: Option<String>,
}

impl LLMProviderSettings {
    /// 检查 Provider 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.url.is_none() && self.key.is_none() && self.model.is_none()
    }
}

/// LLM 配置（TOML）
/// 支持按 provider 分组，每个 provider 有独立的配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMSettings {
    /// 当前使用的 LLM Provider (openai, deepseek, proxy)
    #[serde(default = "LLMSettings::default_provider")]
    pub provider: String,
    /// LLM 输出语言（en, zh, zh-CN, zh-TW 等，默认 en），用于控制 AI 生成内容（如 PR 总结等）的语言
    /// 所有 provider 共享此语言设置
    #[serde(
        default = "LLMSettings::default_language",
        skip_serializing_if = "String::is_empty"
    )]
    pub language: String,
    /// OpenAI 配置
    #[serde(default, skip_serializing_if = "LLMProviderSettings::is_empty")]
    pub openai: LLMProviderSettings,
    /// DeepSeek 配置
    #[serde(default, skip_serializing_if = "LLMProviderSettings::is_empty")]
    pub deepseek: LLMProviderSettings,
    /// Proxy 配置
    #[serde(default, skip_serializing_if = "LLMProviderSettings::is_empty")]
    pub proxy: LLMProviderSettings,
}

impl Default for LLMSettings {
    fn default() -> Self {
        Self {
            provider: Self::default_provider(),
            language: Self::default_language(),
            openai: LLMProviderSettings::default(),
            deepseek: LLMProviderSettings::default(),
            proxy: LLMProviderSettings::default(),
        }
    }
}

impl LLMSettings {
    /// 默认 LLM Provider
    pub fn default_provider() -> String {
        "openai".to_string()
    }

    /// 根据 Provider 获取默认模型
    pub fn default_model(provider: &str) -> String {
        match provider {
            "openai" => "gpt-4.0".to_string(),
            "deepseek" => "deepseek-chat".to_string(),
            _ => String::new(), // proxy 必须输入，没有默认值
        }
    }

    /// 默认 LLM 输出语言
    pub fn default_language() -> String {
        "en".to_string()
    }

    /// 获取当前 provider 的配置
    pub fn current_provider(&self) -> &LLMProviderSettings {
        match self.provider.as_str() {
            "openai" => &self.openai,
            "deepseek" => &self.deepseek,
            "proxy" => &self.proxy,
            _ => &self.openai, // 默认返回 openai
        }
    }

    /// 获取当前 provider 的配置（可变引用）
    pub fn current_provider_mut(&mut self) -> &mut LLMProviderSettings {
        match self.provider.as_str() {
            "openai" => &mut self.openai,
            "deepseek" => &mut self.deepseek,
            "proxy" => &mut self.proxy,
            _ => &mut self.openai, // 默认返回 openai
        }
    }

    /// 检查 LLM 配置是否为空
    fn is_empty(&self) -> bool {
        self.openai.is_empty()
            && self.deepseek.is_empty()
            && self.proxy.is_empty()
            && self.provider == Self::default_provider()
            && self.language == Self::default_language()
    }
}

/// 应用程序设置
/// 从 workflow.toml 配置文件读取配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    /// Jira 配置
    #[serde(default, skip_serializing_if = "JiraSettings::is_empty")]
    pub jira: JiraSettings,
    /// GitHub 配置
    #[serde(default, skip_serializing_if = "GitHubSettings::is_empty")]
    pub github: GitHubSettings,
    /// 日志配置
    #[serde(default, skip_serializing_if = "LogSettings::is_empty")]
    pub log: LogSettings,
    /// LLM 配置
    #[serde(default, skip_serializing_if = "LLMSettings::is_empty")]
    pub llm: LLMSettings,
    /// 别名配置（TOML section: [aliases]）
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub aliases: HashMap<String, String>,
}

impl Settings {
    /// 获取缓存的 Settings 实例
    /// 从 workflow.toml 配置文件加载，如果文件不存在则返回默认值
    pub fn get() -> &'static Settings {
        static SETTINGS: OnceLock<Settings> = OnceLock::new();
        SETTINGS.get_or_init(Self::load)
    }

    /// 从 workflow.toml 配置文件加载设置
    /// 如果配置文件不存在或字段缺失，使用默认值
    pub fn load() -> Self {
        match Paths::workflow_config() {
            Ok(config_path) => {
                if !config_path.exists() {
                    Self::default()
                } else {
                    match FileReader::new(&config_path).to_string() {
                        Ok(content) => toml::from_str::<Self>(&content).unwrap_or_default(),
                        Err(_) => Self::default(),
                    }
                }
            }
            Err(_) => Self::default(),
        }
    }

    /// 检查配置文件权限（仅 Unix 系统）
    /// 返回警告信息（如果有）
    #[cfg(unix)]
    pub fn check_permissions() -> Option<String> {
        if let Ok(config_path) = Paths::workflow_config() {
            if config_path.exists() {
                if let Ok(metadata) = config_path.metadata() {
                    let permissions = metadata.permissions();
                    let mode = permissions.mode();
                    // 检查是否有组或其他用户权限（非 600）
                    if (mode & 0o077) != 0 {
                        return Some(format!(
                            "Warning: Configuration file has overly permissive permissions (current: {:o}). Consider setting to 600 for better security.",
                            mode & 0o777
                        ));
                    }
                }
            }
        }
        None
    }

    /// 检查配置文件权限（非 Unix 系统，总是返回 None）
    #[cfg(not(unix))]
    pub fn check_permissions() -> Option<String> {
        None
    }

    /// 获取所有配置并验证（用于 `workflow config` 命令）
    ///
    /// 获取所有配置项，并对 Jira 和 GitHub 配置进行验证。
    ///
    /// # 返回
    ///
    /// 返回包含所有配置信息的 `VerificationResult`。
    pub fn verify(&self) -> Result<VerificationResult> {
        Ok(VerificationResult {
            log: LogConfigInfo {
                output_folder_name: self.log.get_output_folder_name(),
                download_base_dir: self.log.download_base_dir.clone(),
            },
            llm: self.get_llm_config(),
            jira: self.verify_jira()?,
            github: self.verify_github()?,
        })
    }

    /// 获取 LLM 配置信息
    pub fn get_llm_config(&self) -> LLMConfigInfo {
        let current = self.llm.current_provider();

        // 获取 model（如果有保存的值，否则显示默认值）
        let model = if let Some(ref model) = current.model {
            model.clone()
        } else {
            LLMSettings::default_model(&self.llm.provider)
        };

        // 组合 model 和 URL（仅在 provider 为 "proxy" 时显示 URL）
        let model_display = if self.llm.provider == "proxy" {
            if let Some(ref url) = current.url {
                if !url.is_empty() {
                    format!("{}({})", model, url)
                } else {
                    model
                }
            } else {
                model
            }
        } else {
            model
        };

        // 获取 Key（掩码显示）
        let key = current.key.as_ref().map(|k| k.mask()).unwrap_or_else(|| "-".to_string());

        // 获取 Language（如果有保存的值，否则显示默认值）
        let language = if !self.llm.language.is_empty() {
            self.llm.language.clone()
        } else {
            LLMSettings::default_language()
        };

        LLMConfigInfo {
            provider: self.llm.provider.clone(),
            model: model_display,
            key,
            language,
        }
    }

    /// 验证 Jira 配置并返回结果
    pub fn verify_jira(&self) -> Result<JiraVerificationResult> {
        if let (Some(email), Some(api_token), Some(service_address)) = (
            &self.jira.email,
            &self.jira.api_token,
            &self.jira.service_address,
        ) {
            let config = JiraConfigInfo {
                email: email.clone(),
                service_address: service_address.clone(),
                api_token: api_token.mask(),
            };

            let base_url = format!("{}/rest/api/2", service_address);
            let url = format!("{}/myself", base_url);

            let verification = match HttpClient::global() {
                Ok(client) => {
                    let auth = Authorization::new(email, api_token);
                    let config = RequestConfig::<Value, Value>::new().auth(&auth);
                    match client.get(&url, config) {
                        Ok(response) => {
                            // 使用 ensure_success 统一处理成功/失败检查
                            match response.ensure_success() {
                                Ok(success_response) => {
                                    match success_response.as_json::<JiraUser>() {
                                        Ok(user) => Some(JiraVerificationStatus::Success {
                                            email: email.clone(),
                                            account_id: user.account_id,
                                        }),
                                        Err(e) => Some(JiraVerificationStatus::Failed {
                                            reason: "Failed to parse Jira user response".to_string(),
                                            details: vec![format!("Error: {}", e)],
                                        }),
                                    }
                                }
                                Err(e) => Some(JiraVerificationStatus::Failed {
                                    reason: "Failed to verify Jira configuration".to_string(),
                                    details: vec![
                                        format!("Error: {}", e),
                                        "Please check your Jira service address, email, and API token.".to_string(),
                                    ],
                                }),
                            }
                        }
                        Err(e) => Some(JiraVerificationStatus::Failed {
                            reason: "Failed to verify Jira configuration".to_string(),
                            details: vec![
                                format!("Error: {}", e),
                                "Please check your Jira service address, email, and API token."
                                    .to_string(),
                            ],
                        }),
                    }
                }
                Err(e) => Some(JiraVerificationStatus::Failed {
                    reason: "Failed to create HTTP client".to_string(),
                    details: vec![format!("Error: {}", e)],
                }),
            };

            Ok(JiraVerificationResult {
                configured: true,
                config: Some(config),
                verification,
            })
        } else {
            Ok(JiraVerificationResult {
                configured: false,
                config: None,
                verification: None,
            })
        }
    }

    /// 验证 GitHub 配置并返回结果
    pub fn verify_github(&self) -> Result<GitHubVerificationResult> {
        if self.github.accounts.is_empty() {
            return Ok(GitHubVerificationResult {
                configured: false,
                accounts: Vec::new(),
                summary: GitHubVerificationSummary {
                    total_count: 0,
                    success_count: 0,
                    failed_accounts: Vec::new(),
                },
            });
        }

        let mut success_count = 0;
        let mut failed_accounts = Vec::new();
        let mut account_infos = Vec::new();

        for account in &self.github.accounts {
            let is_current =
                self.github.current.as_ref().map(|c| c == &account.name).unwrap_or_else(|| {
                    // 如果没有设置 current，第一个账号是当前账号
                    self.github.accounts.first().map(|a| &a.name) == Some(&account.name)
                });

            // 使用该账号的 token 验证
            let (verification_status, verification_error) =
                match GitHub::get_user_info(Some(&account.api_token)) {
                    Ok(_user) => {
                        success_count += 1;
                        ("Success".to_string(), None)
                    }
                    Err(e) => {
                        failed_accounts.push(account.name.clone());
                        ("Failed".to_string(), Some(format!("{}", e)))
                    }
                };

            account_infos.push(GitHubAccountInfo {
                name: account.name.clone(),
                is_current,
                email: account.email.clone(),
                token: account.api_token.mask(),
                verification_status,
                verification_error,
            });
        }

        let total_count = self.github.accounts.len();
        Ok(GitHubVerificationResult {
            configured: true,
            accounts: account_infos,
            summary: GitHubVerificationSummary {
                total_count,
                success_count,
                failed_accounts,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::settings::{
        GitHubAccountListRow, GitHubAccountRow, JiraConfigRow, LLMConfigRow,
    };
    use pretty_assertions::assert_eq;

    // ==================== Helper Functions ====================

    /// 创建测试用的 JiraSettings
    fn create_test_jira_settings() -> JiraSettings {
        JiraSettings {
            email: Some("test@example.com".to_string()),
            api_token: Some("test_token_123".to_string()),
            service_address: Some("https://company.atlassian.net".to_string()),
        }
    }

    /// 创建测试用的 GitHubSettings
    fn create_test_github_settings() -> GitHubSettings {
        GitHubSettings {
            accounts: vec![
                GitHubAccount {
                    name: "personal".to_string(),
                    email: "personal@example.com".to_string(),
                    api_token: "ghp_personal_token".to_string(),
                },
                GitHubAccount {
                    name: "work".to_string(),
                    email: "work@company.com".to_string(),
                    api_token: "ghp_work_token".to_string(),
                },
            ],
            current: Some("personal".to_string()),
        }
    }

    /// 创建测试用的 LLMSettings
    fn create_test_llm_settings() -> LLMSettings {
        LLMSettings {
            provider: "openai".to_string(),
            language: "English".to_string(),
            openai: LLMProviderSettings {
                url: None,
                key: Some("sk-test_openai_key".to_string()),
                model: Some("gpt-4".to_string()),
            },
            deepseek: LLMProviderSettings {
                url: None,
                key: Some("sk-test_deepseek_key".to_string()),
                model: Some("deepseek-chat".to_string()),
            },
            proxy: LLMProviderSettings {
                url: Some("https://api.proxy.com".to_string()),
                key: Some("proxy_key".to_string()),
                model: Some("proxy-model".to_string()),
            },
        }
    }

    // ==================== JiraSettings 测试 ====================

    /// 测试 JiraSettings 创建和字段访问
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_jira_settings_creation() {
        let jira_settings = create_test_jira_settings();

        assert_eq!(jira_settings.email, Some("test@example.com".to_string()));
        assert_eq!(jira_settings.api_token, Some("test_token_123".to_string()));
        assert_eq!(
            jira_settings.service_address,
            Some("https://company.atlassian.net".to_string())
        );
    }

    /// 测试 JiraSettings 默认实现
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_jira_settings_default() {
        let default_jira = JiraSettings::default();

        assert_eq!(default_jira.email, None);
        assert_eq!(default_jira.api_token, None);
        assert_eq!(default_jira.service_address, None);
    }

    /// 测试 JiraSettings 克隆和调试输出
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_jira_settings_clone_and_debug() {
        let original_jira = create_test_jira_settings();
        let cloned_jira = original_jira.clone();

        assert_eq!(original_jira.email, cloned_jira.email);
        assert_eq!(original_jira.api_token, cloned_jira.api_token);
        assert_eq!(original_jira.service_address, cloned_jira.service_address);

        // Arrange: 准备测试调试输出
        let debug_str = format!("{:?}", original_jira);
        assert!(debug_str.contains("JiraSettings"));
        assert!(debug_str.contains("test@example.com"));
    }

    // ==================== GitHubSettings Tests ====================

    /// 测试创建GitHubSettings并验证账号信息
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_github_settings_creation_with_valid_accounts_creates_settings() {
        // Arrange: 准备测试用的 GitHubSettings
        let github_settings = create_test_github_settings();

        // Act: 验证设置创建
        // (验证在 Assert 中完成)

        // Assert: 验证账号数量和当前账号设置正确
        assert_eq!(github_settings.accounts.len(), 2);
        assert_eq!(github_settings.current, Some("personal".to_string()));
        let personal_account = &github_settings.accounts[0];
        assert_eq!(personal_account.name, "personal");
        assert_eq!(personal_account.email, "personal@example.com");
        assert_eq!(personal_account.api_token, "ghp_personal_token");
    }

    /// 测试获取GitHubSettings的当前账号和token
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_github_settings_current_account_with_valid_settings_returns_account() -> Result<()> {
        // Arrange: 准备测试用的 GitHubSettings
        let github_settings = create_test_github_settings();

        // Act: 获取当前账号和 token
        let current_account = github_settings.get_current_account();
        let current_token = github_settings.get_current_token();

        // Assert: 验证当前账号和 token 正确
        assert!(current_account.is_some());
        let account = current_account
            .ok_or_else(|| color_eyre::eyre::eyre!("current account should exist"))?;
        assert_eq!(account.name, "personal");
        assert_eq!(account.email, "personal@example.com");
        assert_eq!(current_token, Some("ghp_personal_token"));
        Ok(())
    }

    /// 测试当current为None时返回第一个账号
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_github_settings_no_current_account_with_none_current_returns_first_account(
    ) -> Result<()> {
        // Arrange: 准备 GitHubSettings（current 为 None）
        let mut github_settings = create_test_github_settings();
        github_settings.current = None;

        // Act: 获取当前账号
        let current_account = github_settings.get_current_account();

        // Assert: 验证返回第一个账号
        assert!(current_account.is_some());
        let account = current_account
            .ok_or_else(|| color_eyre::eyre::eyre!("should return first account"))?;
        assert_eq!(account.name, "personal");
        Ok(())
    }

    /// 测试当账号列表为空时返回None
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_github_settings_empty_accounts_with_no_accounts_returns_none() {
        // Arrange: 准备空的 GitHubSettings
        let empty_github = GitHubSettings {
            accounts: vec![],
            current: None,
        };

        // Act: 获取当前账号和 token
        let current_account = empty_github.get_current_account();
        let current_token = empty_github.get_current_token();

        // Assert: 验证返回 None
        assert!(current_account.is_none());
        assert!(current_token.is_none());
    }

    /// 测试创建默认的GitHubSettings
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_github_settings_default_with_no_parameters_creates_empty_settings() {
        // Arrange: 准备创建默认设置

        // Act: 创建默认的 GitHubSettings
        let default_github = GitHubSettings::default();

        // Assert: 验证账号列表为空且当前账号为 None
        assert!(default_github.accounts.is_empty());
        assert_eq!(default_github.current, None);
    }

    // ==================== LLMSettings Tests ====================

    /// 测试创建LLMSettings并验证提供商配置
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_llm_settings_creation_with_valid_providers_creates_settings() {
        // Arrange: 准备测试用的 LLMSettings
        let llm_settings = create_test_llm_settings();

        // Act: 验证设置创建
        // (验证在 Assert 中完成)

        // Assert: 验证提供商和语言设置正确，以及各个提供商配置正确
        assert_eq!(llm_settings.provider, "openai");
        assert_eq!(llm_settings.language, "English");
        assert_eq!(
            llm_settings.openai.key,
            Some("sk-test_openai_key".to_string())
        );
        assert_eq!(llm_settings.openai.model, Some("gpt-4".to_string()));
        assert_eq!(
            llm_settings.deepseek.key,
            Some("sk-test_deepseek_key".to_string())
        );
        assert_eq!(
            llm_settings.proxy.url,
            Some("https://api.proxy.com".to_string())
        );
    }

    /// 测试获取LLMSettings的当前提供商配置
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_llm_settings_current_provider_with_valid_settings_returns_provider() {
        // Arrange: 准备测试用的 LLMSettings
        let llm_settings = create_test_llm_settings();

        // Act: 获取当前提供商
        let current_provider = llm_settings.current_provider();

        // Assert: 验证当前提供商配置正确
        assert_eq!(current_provider.key, Some("sk-test_openai_key".to_string()));
        assert_eq!(current_provider.model, Some("gpt-4".to_string()));
    }

    /// 测试LLMSettings的默认值方法
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_llm_settings_defaults_with_no_parameters_returns_default_values() {
        // Arrange: 准备检查默认值

        // Act & Assert: 验证各个默认值方法返回正确的值
        assert_eq!(LLMSettings::default_provider(), "openai");
        assert_eq!(LLMSettings::default_language(), "en");
        assert_eq!(LLMSettings::default_model("openai"), "gpt-4.0");
        assert_eq!(LLMSettings::default_model("deepseek"), "deepseek-chat");
        assert_eq!(LLMSettings::default_model("unknown"), ""); // proxy 必须输入，没有默认值
    }

    /// 测试创建LLMProviderSettings并验证字段值
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_llm_provider_settings_creation_with_valid_fields_creates_settings() {
        // Arrange: 准备提供商设置字段值
        let url = Some("https://api.example.com".to_string());
        let key = Some("test_key".to_string());
        let model = Some("test_model".to_string());

        // Act: 创建 LLMProviderSettings 实例
        let provider_settings = LLMProviderSettings {
            url: url.clone(),
            key: key.clone(),
            model: model.clone(),
        };

        // Assert: 验证字段值正确
        assert_eq!(provider_settings.url, url);
        assert_eq!(provider_settings.key, key);
        assert_eq!(provider_settings.model, model);

        // Arrange: 准备测试默认值
        let default_provider = LLMProviderSettings::default();
        assert_eq!(default_provider.url, None);
        assert_eq!(default_provider.key, None);
        assert_eq!(default_provider.model, None);
    }

    // ==================== LogSettings 测试 ====================

    /// 测试 LogSettings 创建和默认值
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_log_settings_creation() {
        let log_settings = LogSettings {
            output_folder_name: Some("custom_logs".to_string()),
            download_base_dir: Some("/custom/path".to_string()),
            level: Some("debug".to_string()),
            enable_trace_console: Some(true),
        };

        assert_eq!(log_settings.get_output_folder_name(), "custom_logs");
        assert_eq!(
            log_settings.download_base_dir,
            Some("/custom/path".to_string())
        );
        assert_eq!(log_settings.level, Some("debug".to_string()));
        assert_eq!(log_settings.enable_trace_console, Some(true));
    }

    /// 测试 LogSettings 默认实现
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_log_settings_default() {
        let default_log = LogSettings::default();

        assert_eq!(default_log.get_output_folder_name(), "logs");
        assert_eq!(default_log.output_folder_name, None);
        assert_eq!(default_log.download_base_dir, None);
        assert_eq!(default_log.level, None);
        assert_eq!(default_log.enable_trace_console, None);
    }

    /// 测试 LogSettings 默认方法
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_log_settings_default_methods() {
        assert_eq!(LogSettings::default_log_folder(), "logs");

        // default_download_base_dir_option() returns None (to indicate using default without writing to config)
        let default_base_dir_option = LogSettings::default_download_base_dir_option();
        assert_eq!(default_base_dir_option, None);

        // Check the actual default path function
        let default_base_dir = default_download_base_dir();
        assert!(default_base_dir.contains("Workflow"));
    }

    // ==================== Settings 主结构测试 ====================

    /// 测试 Settings 创建和默认实现
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_settings_creation() {
        let settings = Settings {
            jira: create_test_jira_settings(),
            github: create_test_github_settings(),
            log: LogSettings::default(),
            llm: create_test_llm_settings(),
            aliases: {
                let mut aliases = HashMap::new();
                aliases.insert("st".to_string(), "status".to_string());
                aliases.insert("co".to_string(), "checkout".to_string());
                aliases
            },
        };

        assert!(settings.jira.email.is_some());
        assert_eq!(settings.github.accounts.len(), 2);
        assert_eq!(settings.aliases.len(), 2);
        assert_eq!(settings.aliases.get("st"), Some(&"status".to_string()));
    }

    /// 测试 Settings 默认实现
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_settings_default() {
        let default_settings = Settings::default();

        assert_eq!(default_settings.jira.email, None);
        assert!(default_settings.github.accounts.is_empty());
        assert_eq!(default_settings.log.get_output_folder_name(), "logs");
        assert_eq!(default_settings.llm.provider, "openai");
        assert!(default_settings.aliases.is_empty());
    }

    // ==================== Table Display Structure Tests ====================

    /// 测试表格行结构创建
    ///
    /// ## 测试目的
    /// 验证测试函数能够正确执行预期功能。
    ///
    /// ## 测试场景
    /// 1. 准备测试数据
    /// 2. 执行被测试的操作
    /// 3. 验证结果
    ///
    /// ## 预期结果
    /// - 测试通过，无错误
    #[test]
    fn test_table_row_structures() {
        // Arrange: 准备测试 LLMConfigRow
        let llm_row = LLMConfigRow {
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            key: "sk-****".to_string(),
            language: "English".to_string(),
        };

        assert_eq!(llm_row.provider, "openai");
        assert_eq!(llm_row.model, "gpt-4");

        // Arrange: 准备测试 JiraConfigRow
        let jira_row = JiraConfigRow {
            email: "jira@example.com".to_string(),
            service_address: "https://jira.company.com".to_string(),
            api_token: "****".to_string(),
        };

        assert_eq!(jira_row.email, "jira@example.com");
        assert!(jira_row.service_address.contains("jira.company.com"));

        // Arrange: 准备测试 GitHubAccountRow
        let github_row = GitHubAccountRow {
            name: "personal".to_string(),
            email: "github@example.com".to_string(),
            token: "ghp_****".to_string(),
            status: "Active".to_string(),
            verification: "Success".to_string(),
        };

        assert_eq!(github_row.name, "personal");
        assert_eq!(github_row.status, "Active");

        // Arrange: 准备测试 GitHubAccountListRow
        let github_list_row = GitHubAccountListRow {
            index: "1".to_string(),
            name: "work".to_string(),
            email: "work@company.com".to_string(),
            token: "ghp_****".to_string(),
            status: "Inactive".to_string(),
        };

        assert_eq!(github_list_row.index, "1");
        assert_eq!(github_list_row.status, "Inactive");
    }

    /// 测试复杂配置场景
    ///
    /// ## 测试目的
    /// 验证Settings结构体能够正确处理包含所有配置类型的复杂场景
    ///
    /// ## 测试场景
    /// 1. 创建包含Jira、GitHub、Log、LLM和别名配置的完整Settings
    /// 2. 验证各个配置模块的字段值正确
    /// 3. 验证GitHub当前账号功能
    /// 4. 验证LLM当前提供商功能
    /// 5. 验证别名功能
    ///
    /// ## 预期结果
    /// - 所有配置字段正确设置
    /// - GitHub当前账号功能正常
    /// - LLM当前提供商功能正常
    /// - 别名功能正常
    #[test]
    fn test_complex_configuration_scenario() -> Result<()> {
        // 创建包含所有配置的复杂设置
        let mut aliases = HashMap::new();
        aliases.insert("s".to_string(), "status".to_string());
        aliases.insert("c".to_string(), "commit".to_string());
        aliases.insert("p".to_string(), "push".to_string());

        let complex_settings = Settings {
            jira: JiraSettings {
                email: Some("complex@jira.com".to_string()),
                api_token: Some("complex_jira_token".to_string()),
                service_address: Some("https://complex.atlassian.net".to_string()),
            },
            github: GitHubSettings {
                accounts: vec![
                    GitHubAccount {
                        name: "main".to_string(),
                        email: "main@github.com".to_string(),
                        api_token: "ghp_main_token".to_string(),
                    },
                    GitHubAccount {
                        name: "backup".to_string(),
                        email: "backup@github.com".to_string(),
                        api_token: "ghp_backup_token".to_string(),
                    },
                    GitHubAccount {
                        name: "test".to_string(),
                        email: "test@github.com".to_string(),
                        api_token: "ghp_test_token".to_string(),
                    },
                ],
                current: Some("main".to_string()),
            },
            log: LogSettings {
                output_folder_name: Some("complex_logs".to_string()),
                download_base_dir: Some("/complex/logs/path".to_string()),
                level: Some("info".to_string()),
                enable_trace_console: Some(false),
            },
            llm: LLMSettings {
                provider: "proxy".to_string(),
                language: "Chinese".to_string(),
                openai: LLMProviderSettings {
                    url: None,
                    key: Some("sk-openai_complex".to_string()),
                    model: Some("gpt-4-turbo".to_string()),
                },
                deepseek: LLMProviderSettings {
                    url: None,
                    key: Some("sk-deepseek_complex".to_string()),
                    model: Some("deepseek-coder".to_string()),
                },
                proxy: LLMProviderSettings {
                    url: Some("https://complex.proxy.api.com".to_string()),
                    key: Some("proxy_complex_key".to_string()),
                    model: Some("complex-model".to_string()),
                },
            },
            aliases,
        };

        // Assert: 验证复杂配置的各个方面
        assert!(complex_settings.jira.email.is_some());
        assert_eq!(complex_settings.github.accounts.len(), 3);
        assert_eq!(complex_settings.log.level, Some("info".to_string()));
        assert_eq!(complex_settings.llm.provider, "proxy");
        assert_eq!(complex_settings.aliases.len(), 3);

        // Assert: 验证 GitHub 当前账号功能
        let current_account = complex_settings.github.get_current_account();
        assert!(current_account.is_some());
        let account = current_account
            .ok_or_else(|| color_eyre::eyre::eyre!("current account should exist"))?;
        assert_eq!(account.name, "main");

        // Assert: 验证 LLM 当前提供商功能
        let current_llm = complex_settings.llm.current_provider();
        assert_eq!(
            current_llm.url,
            Some("https://complex.proxy.api.com".to_string())
        );

        // Assert: 验证别名功能
        assert_eq!(
            complex_settings.aliases.get("s"),
            Some(&"status".to_string())
        );
        assert_eq!(
            complex_settings.aliases.get("c"),
            Some(&"commit".to_string())
        );
        assert_eq!(complex_settings.aliases.get("p"), Some(&"push".to_string()));
        Ok(())
    }
}
