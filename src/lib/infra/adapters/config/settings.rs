//! Settings 适配器
//!
//! 将 Settings 适配为配置提供者，实现配置读取的适配器模式。
//! 实现 `logger::LogConfigProvider`、`llm::LLMConfigProvider` 和 `jira::JiraConfigProvider` trait，
//! 使 logger、llm 和 jira 可以通过适配器使用配置。

use crate::jira::JiraConfigProvider;
use crate::llm::client::LLMConfigProvider;
use crate::logger::LogConfigProvider;
use crate::settings::default_download_base_dir;
use crate::settings::paths::Paths;
use crate::settings::{LLMSettings, Settings};
use crate::LogLevel;
use color_eyre::{eyre::WrapErr, Result};
use std::path::PathBuf;

/// Settings 适配器
///
/// 将 `Settings` 适配为配置提供者，使 logger 可以通过适配器使用配置。
pub struct SettingsAdapter {
    settings: &'static Settings,
}

impl SettingsAdapter {
    /// 创建新的适配器实例
    ///
    /// # 返回
    ///
    /// 使用全局 `Settings::get()` 创建的适配器实例。
    pub fn new() -> Self {
        Self {
            settings: Settings::get(),
        }
    }

    /// 从指定的 Settings 实例创建适配器
    ///
    /// # 参数
    ///
    /// * `settings` - Settings 实例的引用
    pub fn from_settings(settings: &'static Settings) -> Self {
        Self { settings }
    }
}

impl Default for SettingsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl LogConfigProvider for SettingsAdapter {
    fn get_log_level(&self) -> Option<LogLevel> {
        self.settings.log.level.as_deref().and_then(|s| s.parse::<LogLevel>().ok())
    }

    fn get_log_format(&self) -> Option<String> {
        self.settings.log.format.clone()
    }

    fn get_enable_console(&self) -> bool {
        self.settings.log.enable_trace_console.unwrap_or(false)
    }

    fn get_logs_dir(&self) -> Result<PathBuf> {
        Paths::logs_dir()
    }
}

impl LLMConfigProvider for SettingsAdapter {
    fn get_provider(&self) -> String {
        self.settings.llm.provider.clone()
    }

    fn get_current_provider_url(&self) -> Option<String> {
        self.settings.llm.current_provider().url.clone()
    }

    fn get_current_provider_key(&self) -> Option<String> {
        self.settings.llm.current_provider().key.clone()
    }

    fn get_current_provider_model(&self) -> Option<String> {
        self.settings.llm.current_provider().model.clone()
    }

    fn get_language(&self) -> String {
        if self.settings.llm.language.is_empty() {
            LLMSettings::default_language()
        } else {
            self.settings.llm.language.clone()
        }
    }
}

impl JiraConfigProvider for SettingsAdapter {
    fn get_jira_email(&self) -> Option<String> {
        self.settings.jira.email.clone()
    }

    fn get_jira_api_token(&self) -> Option<String> {
        self.settings.jira.api_token.clone()
    }

    fn get_jira_service_address(&self) -> Option<String> {
        self.settings.jira.service_address.clone()
    }

    fn get_download_base_dir(&self) -> Result<PathBuf> {
        let base_dir_str = self
            .settings
            .log
            .download_base_dir
            .clone()
            .unwrap_or_else(default_download_base_dir);
        Paths::expand(&base_dir_str)
            .wrap_err_with(|| format!("Failed to expand path: {}", base_dir_str))
    }

    fn get_log_output_folder_name(&self) -> String {
        self.settings.log.get_output_folder_name()
    }

    fn get_jira_config_path(&self) -> Result<PathBuf> {
        Paths::jira_config()
    }

    fn get_work_history_dir(&self) -> Result<PathBuf> {
        Paths::work_history_dir()
    }
}
