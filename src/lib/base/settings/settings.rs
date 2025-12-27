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
