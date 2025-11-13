use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use std::fs;

use super::defaults::{default_download_base_dir_option, default_log_folder, default_log_settings};
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
    /// 操作完成后是否删除日志
    #[serde(default)]
    pub delete_when_completed: bool,
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

/// 代理配置（TOML）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxySettings {
    /// 是否禁用代理检查
    #[serde(default)]
    pub disable_check: bool,
}

/// Codeup 配置（TOML）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeupSettings {
    /// Codeup 项目 ID
    pub project_id: Option<u64>,
    /// Codeup CSRF Token
    pub csrf_token: Option<String>,
    /// Codeup Cookie
    pub cookie: Option<String>,
}

// ==================== TOML LLM 配置结构体 ====================

/// LLM 配置（TOML）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMSettingsToml {
    /// OpenAI API Key
    pub openai_key: Option<String>,
    /// LLM 代理 URL
    pub llm_proxy_url: Option<String>,
    /// LLM 代理 Key
    pub llm_proxy_key: Option<String>,
    /// DeepSeek API Key
    pub deepseek_key: Option<String>,
    /// LLM Provider (openai, deepseek, proxy)
    #[serde(default = "default_llm_provider")]
    pub llm_provider: String,
}

fn default_llm_provider() -> String {
    "openai".to_string()
}

// ==================== 应用配置结构体 ====================

/// LLM 相关设置
#[derive(Debug, Clone)]
pub struct LLMSettings {
    /// OpenAI API Key
    pub openai_key: Option<String>,
    /// LLM 代理 URL
    pub llm_proxy_url: Option<String>,
    /// LLM 代理 Key
    pub llm_proxy_key: Option<String>,
    /// DeepSeek API Key
    pub deepseek_key: Option<String>,
    /// LLM Provider (openai, deepseek, proxy)
    pub llm_provider: String,
}

impl LLMSettings {
    /// 创建默认的 LLMSettings 实例
    fn default() -> Self {
        Self {
            openai_key: None,
            llm_proxy_url: None,
            llm_proxy_key: None,
            deepseek_key: None,
            llm_provider: "openai".to_string(),
        }
    }
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
    /// 代理配置
    #[serde(default)]
    pub proxy: ProxySettings,
    /// Codeup 配置
    #[serde(default)]
    pub codeup: CodeupSettings,
    /// LLM 相关设置（不在 TOML 中，单独加载）
    #[serde(skip)]
    pub llm: Option<LLMSettings>,
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
        let mut settings = match ConfigPaths::workflow_config() {
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
        };

        // 加载 LLM 设置
        let llm_settings = match ConfigPaths::llm_config() {
            Ok(config_path) => {
                if !config_path.exists() {
                    LLMSettings::default()
                } else {
                    match fs::read_to_string(&config_path) {
                        Ok(content) => match toml::from_str::<LLMSettingsToml>(&content) {
                            Ok(toml_config) => LLMSettings {
                                openai_key: toml_config.openai_key,
                                llm_proxy_url: toml_config.llm_proxy_url,
                                llm_proxy_key: toml_config.llm_proxy_key,
                                deepseek_key: toml_config.deepseek_key,
                                llm_provider: toml_config.llm_provider,
                            },
                            Err(_) => LLMSettings::default(),
                        },
                        Err(_) => LLMSettings::default(),
                    }
                }
            }
            Err(_) => LLMSettings::default(),
        };
        settings.llm = Some(llm_settings);

        settings
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
        assert_eq!(settings.llm.as_ref().unwrap().llm_provider, "openai"); // 默认值
    }

    #[test]
    fn test_boolean_flags() {
        // 测试默认值
        let settings = Settings::load();
        assert!(!settings.log.delete_when_completed);
        assert!(!settings.proxy.disable_check);
    }

    #[test]
    fn test_llm_provider() {
        // 测试默认值
        let settings = Settings::load();
        assert_eq!(settings.llm.as_ref().unwrap().llm_provider, "openai");
    }
}
