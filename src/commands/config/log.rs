use crate::base::constants::messages::log;
use crate::base::settings::paths::Paths;
use crate::base::settings::Settings;
use crate::base::LogLevel;
use crate::jira::config::ConfigManager;
use crate::select;
use crate::{br, info, success};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};

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

        // 显示选择菜单
        let log_levels_vec: Vec<&str> = log_levels.to_vec();
        let selected_level_str = select!(
            "Select log level [current: {}]",
            current_level_str,
            log_levels_vec
        )
        .default(current_idx)
        .prompt()
        .wrap_err("Failed to select log level")?;
        let selected_level = selected_level_str.parse::<LogLevel>().map_err(|e| eyre!("{}", e))?;

        // 设置日志级别（内存中）
        LogLevel::set_level(selected_level);

        // 保存到配置文件
        Self::save_log_level_to_config(selected_level_str)?;

        // 显示结果
        br!();
        success!("Log level set to: {}", selected_level_str);
        info!("  Current log level: {}", selected_level.as_str());
        if let Ok(config_path) = crate::base::Paths::workflow_config() {
            info!("  {} {}", log::CONFIG_SAVED_PREFIX, config_path.display());
        } else {
            info!(
                "  {} ~/.workflow/config/workflow.toml",
                log::CONFIG_SAVED_PREFIX
            );
        }

        Ok(())
    }

    /// 检查当前日志级别
    pub fn check() -> Result<()> {
        let current_level = LogLevel::get_level();
        let default_level = LogLevel::default_level();
        let config_level = Settings::get().log.level.as_ref();

        success!("Current log level: {}", current_level.as_str());
        info!(
            "Default log level: {} (based on build mode)",
            default_level.as_str()
        );

        if let Some(level_str) = config_level {
            info!(
                "Config file level: {} (from ~/.workflow/config/workflow.toml)",
                level_str
            );
        } else {
            info!("Config file level: not set (using default)");
        }

        if current_level == default_level && config_level.is_none() {
            info!("Log level is at default (not manually set)");
        } else {
            info!("Log level has been manually set");
        }

        br!();
        info!("Available log levels:");
        info!("  none  - No log output");
        info!("  error - Only error messages");
        info!("  warn  - Warning and error messages");
        info!("  info  - Info, warning, and error messages");
        info!("  debug - All log messages (including debug)");

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

        info!("Current trace console output: {}", current_status);
        br!();

        // 显示选项
        let options = vec![
            "Enable (output to both file and console)",
            "Disable (output to file only)",
        ];

        let current_idx = if current_value { 0 } else { 1 };

        let selected_option = crate::select!("Select trace console output mode", options.clone())
            .default(current_idx)
            .prompt()
            .wrap_err("Failed to select trace console option")?;

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
        br!();
        if selected_idx == 0 {
            success!("Trace console output enabled");
            info!("  Tracing logs will be output to both file and console (stderr)");
            if let Ok(config_path) = crate::base::Paths::workflow_config() {
                info!("  {} {}", log::CONFIG_SAVED_PREFIX, config_path.display());
            } else {
                info!(
                    "  {} ~/.workflow/config/workflow.toml",
                    log::CONFIG_SAVED_PREFIX
                );
            }
        } else {
            success!("Trace console output disabled");
            info!("  Tracing logs will only be output to file");
            info!("  Configuration updated (removed from config file)");
        }

        Ok(())
    }
}
