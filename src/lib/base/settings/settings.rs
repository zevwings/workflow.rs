use std::sync::OnceLock;

use crate::base::util::file::FileReader;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::defaults::default_llm_model;
use crate::base::http::{Authorization, HttpClient, RequestConfig};
use crate::jira::types::JiraUser;
use crate::mask_sensitive_value;
use crate::pr::GitHub;
use std::collections::HashMap;

use super::defaults::{
    default_download_base_dir_option, default_language, default_llm_provider, default_log_folder,
    default_log_settings,
};
use super::paths::Paths;

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

// Codeup 配置信息已移除（Codeup support has been removed）
// #[derive(Debug, Clone)]
// pub struct CodeupConfigInfo {
//     /// 项目 ID
//     pub project_id: Option<u64>,
//     /// CSRF Token（掩码显示）
//     pub csrf_token: Option<String>,
//     /// Cookie（掩码显示）
//     pub cookie: Option<String>,
// }

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
    // /// Codeup 配置  // Codeup support has been removed
    // pub codeup: CodeupConfigInfo,
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

/// 日志配置（TOML）
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSettings {
    /// 日志输出文件夹名称
    #[serde(default = "default_log_folder")]
    pub output_folder_name: String,
    /// 日志下载基础目录
    #[serde(default = "default_download_base_dir_option")]
    pub download_base_dir: Option<String>,
    /// 日志级别（none, error, warn, info, debug）
    pub level: Option<String>,
    /// 是否同时输出 tracing 日志到控制台（stderr）
    /// 如果为 `true`，tracing 日志会同时输出到文件和控制台
    /// 如果配置文件中不存在此字段，默认为 `false`（只输出到文件）
    /// 注意：只有设置为 `true` 时才会写入配置文件，设置为 `false` 时从配置文件中删除
    pub enable_trace_console: Option<bool>,
}

impl Default for LogSettings {
    fn default() -> Self {
        default_log_settings()
    }
}

// Codeup 配置已移除（Codeup support has been removed）
// #[derive(Debug, Clone, Default, Serialize, Deserialize)]
// pub struct CodeupSettings {
//     /// Codeup 项目 ID
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub project_id: Option<u64>,
//     /// Codeup CSRF Token
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub csrf_token: Option<String>,
//     /// Codeup Cookie
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub cookie: Option<String>,
// }
//
// impl CodeupSettings {
//     /// 检查 Codeup 配置是否为空（所有字段都是 None）
//     fn is_empty(&self) -> bool {
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
    #[serde(default = "default_llm_provider")]
    pub provider: String,
    /// LLM 输出语言（en, zh, zh-CN, zh-TW 等，默认 en），用于控制 AI 生成内容（如 PR 总结等）的语言
    /// 所有 provider 共享此语言设置
    #[serde(default = "default_language", skip_serializing_if = "String::is_empty")]
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
            provider: default_llm_provider(),
            language: default_language(),
            openai: LLMProviderSettings::default(),
            deepseek: LLMProviderSettings::default(),
            proxy: LLMProviderSettings::default(),
        }
    }
}

impl LLMSettings {
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
            && self.provider == default_llm_provider()
            && self.language == default_language()
    }

    /// 从旧格式迁移配置（向后兼容）
    /// 如果配置文件中存在旧的格式（url, key, model 在顶层），则迁移到新格式
    pub fn migrate_from_old_format(
        old_provider: &str,
        old_url: &Option<String>,
        old_key: &Option<String>,
        old_model: &Option<String>,
    ) -> Self {
        let mut new = LLMSettings {
            provider: old_provider.to_string(),
            ..Default::default()
        };

        // 根据 provider 将旧配置迁移到对应的 provider 配置块
        match old_provider {
            "openai" => {
                new.openai.key = old_key.clone();
                new.openai.model = old_model.clone();
            }
            "deepseek" => {
                new.deepseek.key = old_key.clone();
                new.deepseek.model = old_model.clone();
            }
            "proxy" => {
                new.proxy.url = old_url.clone();
                new.proxy.key = old_key.clone();
                new.proxy.model = old_model.clone();
            }
            _ => {}
        }

        new
    }
}

/// 应用程序设置
/// 从 workflow.toml 配置文件读取配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    /// Jira 配置
    #[serde(default)]
    pub jira: JiraSettings,
    /// GitHub 配置
    #[serde(default)]
    pub github: GitHubSettings,
    /// 日志配置
    #[serde(default)]
    pub log: LogSettings,
    // /// Codeup 配置  // Codeup support has been removed
    // #[serde(default, skip_serializing_if = "CodeupSettings::is_empty")]
    // pub codeup: CodeupSettings,
    /// LLM 配置
    #[serde(default, skip_serializing_if = "LLMSettings::is_empty")]
    pub llm: LLMSettings,
    /// 别名配置（TOML section: [aliases]）
    #[serde(default)]
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
    /// 支持从旧格式（扁平结构）自动迁移到新格式（按 provider 分组）
    pub fn load() -> Self {
        match Paths::workflow_config() {
            Ok(config_path) => {
                if !config_path.exists() {
                    Self::default()
                } else {
                    match FileReader::read_to_string(&config_path) {
                        Ok(content) => {
                            // 先尝试解析为新格式
                            match toml::from_str::<Self>(&content) {
                                Ok(settings) => settings,
                                Err(_) => {
                                    // 如果解析失败，尝试解析为旧格式并迁移
                                    #[derive(Debug, Default, Deserialize)]
                                    struct OldLLMSettings {
                                        #[serde(default = "default_llm_provider")]
                                        provider: String,
                                        url: Option<String>,
                                        key: Option<String>,
                                        model: Option<String>,
                                        #[serde(default = "default_language")]
                                        language: String,
                                    }

                                    #[derive(Debug, Deserialize)]
                                    struct OldSettings {
                                        #[serde(default)]
                                        jira: JiraSettings,
                                        #[serde(default)]
                                        github: GitHubSettings,
                                        #[serde(default)]
                                        log: LogSettings,
                                        #[serde(default)]
                                        llm: OldLLMSettings,
                                    }

                                    match toml::from_str::<OldSettings>(&content) {
                                        Ok(old_settings) => {
                                            // 迁移旧格式到新格式
                                            let mut llm = LLMSettings::migrate_from_old_format(
                                                &old_settings.llm.provider,
                                                &old_settings.llm.url,
                                                &old_settings.llm.key,
                                                &old_settings.llm.model,
                                            );
                                            llm.language = old_settings.llm.language;
                                            Settings {
                                                jira: old_settings.jira,
                                                github: old_settings.github,
                                                log: old_settings.log,
                                                llm,
                                                aliases: HashMap::new(),
                                            }
                                        }
                                        Err(_) => Self::default(),
                                    }
                                }
                            }
                        }
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
                output_folder_name: self.log.output_folder_name.clone(),
                download_base_dir: self.log.download_base_dir.clone(),
            },
            llm: self.get_llm_config(),
            // codeup: CodeupConfigInfo {  // Codeup support has been removed
            //     project_id: self.codeup.project_id,
            //     csrf_token: self
            //         .codeup
            //         .csrf_token
            //         .as_ref()
            //         .map(|t| mask_sensitive_value(t)),
            //     cookie: self.codeup.cookie.as_ref().map(|c| mask_sensitive_value(c)),
            // },
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
            default_llm_model(&self.llm.provider)
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
        let key = current
            .key
            .as_ref()
            .map(|k| mask_sensitive_value(k))
            .unwrap_or_else(|| "-".to_string());

        // 获取 Language（如果有保存的值，否则显示默认值）
        let language = if !self.llm.language.is_empty() {
            self.llm.language.clone()
        } else {
            default_language()
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
                api_token: mask_sensitive_value(api_token),
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
                token: mask_sensitive_value(&account.api_token),
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
