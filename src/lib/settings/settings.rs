use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use std::fs;

use super::defaults::{
    default_download_base_dir_option, default_llm_provider, default_log_folder,
    default_log_settings, default_response_format,
};
use super::paths::ConfigPaths;

// ==================== TOML 配置结构体 ====================

/// 用户配置（TOML）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserSettings {
    /// 用户邮箱
    pub email: Option<String>,
}

/// Jira 配置（TOML）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JiraSettings {
    /// Jira API Token
    pub api_token: Option<String>,
    /// Jira 服务地址
    pub service_address: Option<String>,
}

/// GitHub 配置（TOML）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitHubSettings {
    /// GitHub 分支前缀
    pub branch_prefix: Option<String>,
    /// GitHub API Token
    pub api_token: Option<String>,
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
    /// 用户配置
    #[serde(default)]
    pub user: UserSettings,
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
        match ConfigPaths::workflow_config() {
            Ok(config_path) => {
                if !config_path.exists() {
                    Self::default()
                } else {
                    match fs::read_to_string(&config_path) {
                        Ok(content) => toml::from_str::<Self>(&content).unwrap_or_default(),
                        Err(_) => Self::default(),
                    }
                }
            }
            Err(_) => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_initialization() {
        // 测试初始化（使用默认值）
        let settings = Settings::load();
        assert_eq!(settings.user.email, None);
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
