use crate::{log_break, log_message, log_success, LogLevel, Settings};
use anyhow::{Context, Result};
use dialoguer::Select;
use std::fs;
use toml;

/// 日志级别管理命令
pub struct LogCommand;

impl LogCommand {
    /// 设置日志级别（交互式选择）
    pub fn set() -> Result<()> {
        // 获取当前日志级别
        let current_level = LogLevel::get_level();

        // 定义日志级别选项
        let log_levels = vec!["none", "error", "warn", "info", "debug"];

        // 找到当前级别的索引
        let current_level_str = current_level.as_str();
        let current_idx = log_levels
            .iter()
            .position(|&level| level == current_level_str)
            .unwrap_or(2); // 默认为 info

        // 创建提示信息
        let prompt = format!(
            "Select log level [current: {}]",
            current_level_str
        );

        // 显示选择菜单
        let selected_idx = Select::new()
            .with_prompt(&prompt)
            .items(&log_levels)
            .default(current_idx)
            .interact()
            .context("Failed to select log level")?;

        // 获取选中的级别
        let selected_level_str = log_levels[selected_idx];
        let selected_level = selected_level_str
            .parse::<LogLevel>()
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        // 设置日志级别（内存中）
        LogLevel::set_level(selected_level);

        // 保存到配置文件
        Self::save_log_level_to_config(selected_level_str)?;

        // 显示结果
        log_break!();
        log_success!("Log level set to: {}", selected_level_str);
        log_message!("  Current log level: {}", selected_level.as_str());
        log_message!("  Configuration saved to ~/.workflow/config/workflow.toml");

        Ok(())
    }

    /// 检查当前日志级别
    pub fn check() -> Result<()> {
        let current_level = LogLevel::get_level();
        let default_level = LogLevel::default_level();
        let config_level = Settings::get().log.level.as_ref();

        log_success!("Current log level: {}", current_level.as_str());
        log_message!("Default log level: {} (based on build mode)", default_level.as_str());

        if let Some(level_str) = config_level {
            log_message!("Config file level: {} (from ~/.workflow/config/workflow.toml)", level_str);
        } else {
            log_message!("Config file level: not set (using default)");
        }

        if current_level == default_level && config_level.is_none() {
            log_message!("Log level is at default (not manually set)");
        } else {
            log_message!("Log level has been manually set");
        }

        log_break!();
        log_message!("Available log levels:");
        log_message!("  none  - No log output");
        log_message!("  error - Only error messages");
        log_message!("  warn  - Warning and error messages");
        log_message!("  info  - Info, warning, and error messages");
        log_message!("  debug - All log messages (including debug)");

        Ok(())
    }

    /// 保存日志级别到配置文件
    fn save_log_level_to_config(level: &str) -> Result<()> {
        use crate::settings::paths::ConfigPaths;
        use crate::settings::settings::LogSettings;

        // 读取现有配置
        let existing_settings = Settings::get().clone();

        // 构建新的配置，更新 log.level
        let updated_settings = Settings {
            user: existing_settings.user,
            jira: existing_settings.jira,
            github: existing_settings.github,
            log: LogSettings {
                output_folder_name: existing_settings.log.output_folder_name,
                download_base_dir: existing_settings.log.download_base_dir,
                level: Some(level.to_string()),
            },
            codeup: existing_settings.codeup,
            llm: existing_settings.llm,
        };

        // 保存到 workflow.toml
        let workflow_config_path = ConfigPaths::workflow_config()?;
        let toml_content = toml::to_string_pretty(&updated_settings)
            .context("Failed to serialize settings to TOML")?;
        fs::write(&workflow_config_path, toml_content)
            .context("Failed to write workflow.toml")?;

        Ok(())
    }
}

