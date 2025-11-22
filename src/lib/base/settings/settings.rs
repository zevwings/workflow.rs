use std::sync::OnceLock;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

use super::defaults::default_llm_model;
use crate::base::http::{Authorization, HttpClient, RequestConfig};
use crate::jira::types::JiraUser;
use crate::pr::GitHub;
use crate::{log_break, log_info, log_message, log_success, log_warning, mask_sensitive_value};

use super::defaults::{
    default_download_base_dir_option, default_llm_provider, default_log_folder,
    default_log_settings, default_response_format,
};
use super::paths::Paths;

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
    /// 分支前缀（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_prefix: Option<String>,
}

/// GitHub 配置（TOML）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitHubSettings {
    /// 多个 GitHub 账号列表
    #[serde(default)]
    pub accounts: Vec<GitHubAccount>,
    /// 当前激活的账号名称
    #[serde(skip_serializing_if = "Option::is_none")]
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

    /// 获取当前账号的分支前缀
    pub fn get_current_branch_prefix(&self) -> Option<&str> {
        self.get_current_account()
            .and_then(|acc| acc.branch_prefix.as_deref())
    }
}

/// 日志配置（TOML）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSettings {
    /// 日志输出文件夹名称
    #[serde(default = "default_log_folder")]
    pub output_folder_name: String,
    /// 日志下载基础目录
    #[serde(default = "default_download_base_dir_option")]
    pub download_base_dir: Option<String>,
    /// 日志级别（none, error, warn, info, debug）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
}

impl Default for LogSettings {
    fn default() -> Self {
        default_log_settings()
    }
}

/// Codeup 配置（TOML）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeupSettings {
    /// Codeup 项目 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<u64>,
    /// Codeup CSRF Token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csrf_token: Option<String>,
    /// Codeup Cookie
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookie: Option<String>,
}

impl CodeupSettings {
    /// 检查 Codeup 配置是否为空（所有字段都是 None）
    fn is_empty(&self) -> bool {
        self.project_id.is_none() && self.csrf_token.is_none() && self.cookie.is_none()
    }
}

// ==================== TOML LLM 配置结构体 ====================

/// LLM 配置（TOML）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LLMSettings {
    /// LLM Provider URL（proxy 时使用，openai/deepseek 时自动设置）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// LLM Provider Key（proxy/openai/deepseek 时使用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// LLM Provider (openai, deepseek, proxy)
    #[serde(default = "default_llm_provider")]
    pub provider: String,
    /// LLM 模型名称（openai: 默认 gpt-4.0, deepseek: 默认 deepseek-chat, proxy: 必填）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// 响应格式路径（用于从响应中提取内容）
    #[serde(
        default = "default_response_format",
        skip_serializing_if = "String::is_empty"
    )]
    pub response_format: String,
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
    /// Codeup 配置
    #[serde(default, skip_serializing_if = "CodeupSettings::is_empty")]
    pub codeup: CodeupSettings,
    /// LLM 配置
    #[serde(default, skip_serializing_if = "LLMSettings::is_empty")]
    pub llm: LLMSettings,
}

impl LLMSettings {
    /// 检查 LLM 配置是否为空
    fn is_empty(&self) -> bool {
        self.url.is_none()
            && self.key.is_none()
            && self.model.is_none()
            && self.provider == default_llm_provider()
            && self.response_format == default_response_format()
    }
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
                    // 检查文件权限（仅 Unix 系统）
                    #[cfg(unix)]
                    {
                        if let Ok(metadata) = config_path.metadata() {
                            use std::os::unix::fs::PermissionsExt;
                            let permissions = metadata.permissions();
                            let mode = permissions.mode();
                            // 检查是否有组或其他用户权限（非 600）
                            if (mode & 0o077) != 0 {
                                log_warning!(
                                    "Warning: Configuration file has overly permissive permissions (current: {:o}). Consider setting to 600 for better security.",
                                    mode & 0o777
                                );
                            }
                        }
                    }

                    match fs::read_to_string(&config_path) {
                        Ok(content) => toml::from_str::<Self>(&content).unwrap_or_default(),
                        Err(_) => Self::default(),
                    }
                }
            }
            Err(_) => Self::default(),
        }
    }

    /// 显示所有配置并验证（用于 `workflow config` 命令）
    ///
    /// 显示所有配置项，并对 Jira 和 GitHub 配置进行验证。
    ///
    /// # 返回
    ///
    /// 如果成功返回 `Ok(())`，否则返回错误。
    pub fn verify(&self) -> Result<()> {
        self.print_log();
        self.print_llm();
        self.verify_codeup();
        self.verify_jira()?;
        self.verify_github()?;
        Ok(())
    }

    /// 显示日志配置
    fn print_log(&self) {
        log_message!("Log Output Folder Name: {}", self.log.output_folder_name);
        if let Some(ref dir) = self.log.download_base_dir {
            log_message!("Download Base Dir: {}", dir);
        }
    }

    /// 显示 LLM 配置
    fn print_llm(&self) {
        log_message!("\nLLM Provider: {}", self.llm.provider);
        if let Some(ref url) = self.llm.url {
            log_message!("LLM URL: {}", url);
        }
        if let Some(ref key) = self.llm.key {
            log_message!("LLM Key: {}", mask_sensitive_value(key));
        }
        // 显示 model（如果有保存的值，否则显示默认值）
        if let Some(ref model) = self.llm.model {
            log_message!("LLM Model: {}", model);
        } else {
            let default_model = default_llm_model(&self.llm.provider);
            log_message!("LLM Model: {} (default)", default_model);
        }
        // 显示 response_format（如果有保存的值，否则显示默认值）
        if self.llm.response_format.is_empty() {
            let default_format = default_response_format();
            log_message!("LLM Response Format: {} (default)", default_format);
        } else {
            log_message!("LLM Response Format: {}", self.llm.response_format);
        }
    }

    /// 显示 Codeup 配置
    fn verify_codeup(&self) {
        if let Some(id) = self.codeup.project_id {
            log_message!("\nCodeup Project ID: {}", id);
        }
        if let Some(ref token) = self.codeup.csrf_token {
            log_message!("Codeup CSRF Token: {}", mask_sensitive_value(token));
        }
        if let Some(ref cookie) = self.codeup.cookie {
            log_message!("Codeup Cookie: {}", mask_sensitive_value(cookie));
        }
    }

    /// 显示并验证 Jira 配置
    fn verify_jira(&self) -> Result<()> {
        if let (Some(email), Some(api_token), Some(service_address)) = (
            &self.jira.email,
            &self.jira.api_token,
            &self.jira.service_address,
        ) {
            log_break!();
            log_info!("Verifying Jira configuration...");
            log_message!("  Email: {}", email);
            log_message!("  Service Address: {}", service_address);
            log_message!("  API Token: {}", mask_sensitive_value(api_token));

            let base_url = format!("{}/rest/api/2", service_address);
            let url = format!("{}/myself", base_url);

            match HttpClient::global() {
                Ok(client) => {
                    let auth = Authorization::new(email, api_token);
                    let config = RequestConfig::<Value, Value>::new().auth(&auth);
                    match client.get(&url, config) {
                        Ok(response) => {
                            if response.is_success() {
                                match response.as_json::<JiraUser>() {
                                    Ok(user) => {
                                        log_success!(
                                            "Jira verified successfully! Email: {} (Account ID: {})",
                                            email,
                                            user.account_id
                                        );
                                    }
                                    Err(e) => {
                                        log_warning!("Failed to parse Jira user response");
                                        log_message!("  Error: {}", e);
                                    }
                                }
                            } else {
                                log_warning!("Failed to verify Jira configuration");
                                log_message!("  Status: {}", response.status);
                                log_message!(
                                    "  Please check your Jira service address, email, and API token."
                                );
                            }
                        }
                        Err(e) => {
                            log_warning!("Failed to verify Jira configuration");
                            log_message!("  Error: {}", e);
                            log_message!(
                                "  Please check your Jira service address, email, and API token."
                            );
                        }
                    }
                }
                Err(e) => {
                    log_warning!("Failed to create HTTP client");
                    log_message!("  Error: {}", e);
                }
            }
        }
        Ok(())
    }

    /// 显示并验证 GitHub 配置
    fn verify_github(&self) -> Result<()> {
        if !self.github.accounts.is_empty() {
            log_break!();
            log_info!("Verifying GitHub configuration...");
            let mut success_count = 0;
            let mut failed_accounts = Vec::new();

            for account in &self.github.accounts {
                let is_current = self
                    .github
                    .current
                    .as_ref()
                    .map(|c| c == &account.name)
                    .unwrap_or_else(|| {
                        // 如果没有设置 current，第一个账号是当前账号
                        self.github.accounts.first().map(|a| &a.name) == Some(&account.name)
                    });
                let current_marker = if is_current { " (current)" } else { "" };
                log_message!("  - {}{}", account.name, current_marker);
                log_message!("    Email: {}", account.email);
                log_message!(
                    "    API Token: {}",
                    mask_sensitive_value(&account.api_token)
                );
                if let Some(ref prefix) = account.branch_prefix {
                    log_message!("    Branch Prefix: {}", prefix);
                }

                // 使用该账号的 token 验证
                match GitHub::get_user_info(Some(&account.api_token)) {
                    Ok(user) => {
                        log_success!("GitHub account '{}' verified successfully!", user.login);
                        success_count += 1;
                    }
                    Err(e) => {
                        log_warning!("Failed to verify account '{}'", account.name);
                        log_message!("  Error: {}", e);
                        log_message!("  Please check your GitHub API token.");
                        failed_accounts.push(account.name.clone());
                    }
                }
            }

            // 显示验证总结
            let total_count = self.github.accounts.len();
            if failed_accounts.is_empty() {
                log_success!(
                    "All {} GitHub account(s) verified successfully!",
                    total_count
                );
            } else {
                log_warning!(
                    "GitHub verification completed: {}/{} account(s) verified successfully",
                    success_count,
                    total_count
                );
                if !failed_accounts.is_empty() {
                    log_message!("  Failed accounts: {}", failed_accounts.join(", "));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_initialization() {
        // 测试初始化（使用默认值）
        let settings = Settings::load();
        assert_eq!(settings.jira.email, None);
        assert_eq!(settings.jira.api_token, None);
        assert_eq!(settings.jira.service_address, None);
        assert_eq!(settings.log.output_folder_name, "logs");
        assert_eq!(settings.llm.provider, "openai"); // 默认值
    }

    #[test]
    fn test_llm_provider() {
        // 测试默认值
        let settings = Settings::load();
        assert_eq!(settings.llm.provider, "openai");
    }
}
