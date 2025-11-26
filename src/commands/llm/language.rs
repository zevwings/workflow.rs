//! LLM 语言设置命令
//! 交互式设置 PR 总结的语言

use crate::base::settings::paths::Paths;
use crate::base::settings::settings::Settings;
use crate::commands::config::helpers::select_language;
use crate::jira::config::ConfigManager;
use crate::{log_break, log_info, log_success};
use anyhow::{Context, Result};

/// LLM 语言设置命令
pub struct LLMLanguageCommand;

impl LLMLanguageCommand {
    /// 设置 PR 总结语言
    pub fn set() -> Result<()> {
        log_break!('=', 40, "Set Summary Language");
        log_break!();

        // 加载当前配置
        let settings = Settings::load();
        let current_language = if !settings.llm.language.is_empty() {
            Some(settings.llm.language.as_str())
        } else {
            None
        };

        // 显示当前语言
        if let Some(lang) = current_language {
            log_info!("Current language: {}", lang);
        } else {
            log_info!("Current language: en (default)");
        }

        log_break!();

        // 交互式选择语言
        let selected_language =
            select_language(current_language).context("Failed to select language")?;

        // 保存到配置文件
        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path);

        manager.update(|settings| {
            settings.llm.language = selected_language.clone();
        })?;

        log_break!();
        log_success!("Summary language set to: {}", selected_language);

        Ok(())
    }
}
