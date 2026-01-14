//! 应用程序设置
//!
//! 从 workflow.toml 配置文件读取配置

use std::collections::HashMap;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::sync::OnceLock;

use crate::util::file::FileReader;
use color_eyre::Result;
use serde::{Deserialize, Serialize};

use super::github::GitHubSettings;
use super::jira::JiraSettings;
use super::paths::Paths;

pub use super::github::GitHubAccount;
pub use super::jira::{JiraConfigInfo, JiraVerificationResult, JiraVerificationStatus};
pub use super::llm::{LLMConfigInfo, LLMSettings};
pub use super::log::{default_download_base_dir, LogSettings};

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

    /// 获取 LLM 配置信息（不进行验证）
    pub fn get_llm_config(&self) -> LLMConfigInfo {
        self.llm.get_llm_config()
    }

    /// 验证 Jira 配置并返回结果
    pub fn verify_jira(&self) -> Result<JiraVerificationResult> {
        self.jira.verify()
    }

    /// 验证 GitHub 配置并返回结果
    pub fn verify_github(&self) -> Result<super::github::GitHubVerificationResult> {
        self.github.verify()
    }
}
