use crate::base::dialog::SelectDialog;
use crate::base::settings::paths::Paths;
use crate::base::settings::settings::Settings;
use crate::base::LogLevel;
use crate::jira::config::ConfigManager;
use crate::{log_break, log_message, log_success};
use anyhow::{Context, Result};

/// 日志级别管理命令
pub struct LogCommand;

impl LogCommand {
    /// 设置日志级别（交互式选择）
    pub fn set() -> Result<()> {
        // 获取当前日志级别
        let current_level = LogLevel::get_level();

        // 定义日志级别选项
        let log_levels = ["off", "error", "warn", "info", "debug"];

        // 找到当前级别的索引
        let current_level_str = current_level.as_str();
        let current_idx =
            log_levels.iter().position(|&level| level == current_level_str).unwrap_or(2); // 默认为 info

        // 创建提示信息
        let prompt = format!("Select log level [current: {}]", current_level_str);

        // 显示选择菜单
        let log_levels_vec: Vec<&str> = log_levels.to_vec();
        let selected_level_str = SelectDialog::new(&prompt, log_levels_vec)
            .with_default(current_idx)
            .prompt()
            .context("Failed to select log level")?;
        let selected_level =
            selected_level_str.parse::<LogLevel>().map_err(|e| anyhow::anyhow!("{}", e))?;

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
        log_message!(
            "Default log level: {} (based on build mode)",
            default_level.as_str()
        );

        if let Some(level_str) = config_level {
            log_message!(
                "Config file level: {} (from ~/.workflow/config/workflow.toml)",
                level_str
            );
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
        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path);
        manager.update(|settings| {
            settings.log.level = Some(level.to_string());
        })?;
        Ok(())
    }

    /// 管理 tracing 控制台输出（交互式选择）
    pub fn trace_console() -> Result<()> {
        let settings = Settings::get();
        let current_value = settings.log.enable_trace_console.unwrap_or(false);

        // 显示当前状态
        let current_status = if current_value {
            "enabled (output to both file and console)"
        } else {
            "disabled (output to file only)"
        };

        log_message!("Current trace console output: {}", current_status);
        log_break!();

        // 显示选项
        let options = vec![
            "Enable (output to both file and console)",
            "Disable (output to file only)",
        ];

        let current_idx = if current_value { 0 } else { 1 };

        let selected_option =
            SelectDialog::new("Select trace console output mode", options.clone())
                .with_default(current_idx)
                .prompt()
                .context("Failed to select trace console option")?;

        let selected_idx = options.iter().position(|&opt| opt == selected_option).unwrap_or(1);

        // 保存到配置文件
        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path);
        manager.update(|settings| {
            // true 时写入配置文件，false 时从配置文件中删除（设置为 None）
            settings.log.enable_trace_console = if selected_idx == 0 {
                Some(true)
            } else {
                None // false 时不写入配置文件
            };
        })?;

        // 显示结果
        log_break!();
        if selected_idx == 0 {
            log_success!("Trace console output enabled");
            log_message!("  Tracing logs will be output to both file and console (stderr)");
            log_message!("  Configuration saved to ~/.workflow/config/workflow.toml");
        } else {
            log_success!("Trace console output disabled");
            log_message!("  Tracing logs will only be output to file");
            log_message!("  Configuration updated (removed from config file)");
        }

        Ok(())
    }
}
