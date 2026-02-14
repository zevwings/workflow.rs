//! 日志工作流阶段 (v2)

use std::error::Error;

use domain::{GlobalConfig, VerificationService};
use prompt::{br, info, separator, ConfirmFormField, FormBuilder, SelectFormField};

use crate::interactive::{
    core::{context::WorkflowContext, stage::WorkflowStage},
    display::VerificationResultFormatter,
};

/// 日志工作流阶段
pub struct LogStage;

impl LogStage {
    /// 构建并运行日志配置表单
    fn run_form(
        settings: &GlobalConfig,
        current_level: &str,
    ) -> Result<(bool, String, bool), String> {
        let has_level = settings.log.level.is_some();
        let default_enable_logging = has_level;

        if let Some(level) = &settings.log.level {
            info!("Log configuration detected!");
            info!("  - Log level: {}", level);
            br!();
        }

        // 日志级别选项
        let level_options = vec![
            "error".to_string(),
            "warn".to_string(),
            "info".to_string(),
            "debug".to_string(),
        ];

        // 计算默认索引
        let mut default_index = 2; // info
        for (i, lvl) in level_options.iter().enumerate() {
            if lvl == &current_level.to_lowercase() {
                default_index = i;
                break;
            }
        }

        // 分隔符
        separator!('─', 80, "Log configuration");
        br!();

        // 构建表单
        let builder = FormBuilder::new()
            .add_confirm(
                ConfirmFormField::new("enable_logging", "Enable logging?")
                    .default(default_enable_logging)
                    .result_title("Enable logging"),
            )
            .add_confirm(
                ConfirmFormField::new("enable_console", "Enable console output for trace logs?")
                    .default(settings.log.enable_trace_console.unwrap_or(false))
                    .result_title("Enable console output")
                    .condition(Box::new(|result| result.get_bool("enable_logging"))),
            )
            .add_select(
                SelectFormField::new(
                    "log_level",
                    format!("Please select your log level [current: {}]", current_level),
                    level_options.clone(),
                )
                .default(default_index)
                .result_title("Your log level")
                .condition(Box::new(|result| result.get_bool("enable_logging"))),
            );

        let result = builder.run().map_err(|e| e.to_string())?;

        // 提取结果
        let enable_logging = result
            .get_raw("enable_logging")
            .and_then(|v| v.downcast_ref::<bool>())
            .copied()
            .unwrap_or(default_enable_logging);

        let enable_console = result
            .get_raw("enable_console")
            .and_then(|v| v.downcast_ref::<bool>())
            .copied()
            .unwrap_or(false);

        let selected_index = result.get_int("log_level");
        let selected_level = level_options
            .get(selected_index)
            .cloned()
            .unwrap_or_else(|| current_level.to_string());

        Ok((enable_logging, selected_level, enable_console))
    }
}

impl WorkflowStage for LogStage {
    fn stage_name(&self) -> &'static str {
        "Log"
    }

    fn configure(&self, context: &mut WorkflowContext) -> Result<(), Box<dyn Error>> {
        let settings = context.settings_mut();

        let current_level = settings.log.level.clone().unwrap_or_else(|| "info".to_string());

        // 构建并运行表单
        let (enable_logging, selected_level, enable_console) =
            Self::run_form(settings, &current_level)?;

        // 更新设置
        if !enable_logging {
            settings.log.level = None;
            settings.log.enable_trace_console = None;
        } else {
            settings.log.level = Some(selected_level);
            settings.log.enable_trace_console = if enable_console { Some(true) } else { None };
        }

        Ok(())
    }

    fn is_configured(&self, _settings: &GlobalConfig) -> bool {
        // 对于日志，如果能加载配置则认为已配置
        // 原始实现总是验证
        true
    }

    fn needs_spinner(&self) -> bool {
        false
    }

    fn verify(
        &self,
        service: &dyn VerificationService,
    ) -> Result<Box<dyn VerificationResultFormatter>, Box<dyn Error>> {
        service
            .verify_log_config()
            .map(|r| Box::new(r) as Box<dyn VerificationResultFormatter>)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

/// 获取日志阶段实例
pub fn log_stage() -> &'static dyn WorkflowStage {
    &LogStage
}
