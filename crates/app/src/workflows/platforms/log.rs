//! Log Workflow Stage (v2)

use crate::workflows::core::context::WorkflowContext;
use crate::workflows::core::stage::WorkflowStage;
use crate::workflows::display::VerificationResultFormatter;
use domain::{GlobalConfig, VerificationService};
use prompt::{br, info, separator, ConfirmFormField, FormBuilder, SelectFormField};
use std::error::Error;

/// The Log workflow stage.
pub struct LogStage;

impl LogStage {
    /// Build and run the log configuration form.
    fn run_form(
        settings: &GlobalConfig,
        current_level: &str,
    ) -> Result<(bool, String, bool), String> {
        let has_level = settings.log.level.is_some();
        let default_enable_logging = has_level;

        if has_level {
            info!("Log configuration is detected!");
            info!("  - Log Level: {}", settings.log.level.as_ref().unwrap());
            br!();
        }

        // Log level options
        let level_options = vec![
            "error".to_string(),
            "warn".to_string(),
            "info".to_string(),
            "debug".to_string(),
        ];

        // Calculate default index
        let mut default_index = 2; // info
        for (i, lvl) in level_options.iter().enumerate() {
            if lvl == &current_level.to_lowercase() {
                default_index = i;
                break;
            }
        }

        // Separator
        separator!('─', 80, "Log Configuration");
        br!();

        // Build form
        let builder = FormBuilder::new()
            .add_confirm(
                ConfirmFormField::new("enable_logging", "Do you want to enable logging?")
                    .default(default_enable_logging)
                    .result_title("Enable logging"),
            )
            .add_confirm(
                ConfirmFormField::new(
                    "enable_console",
                    "Do you want to enable console output for tracing logs?",
                )
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

        // Extract results
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

        let current_level = settings
            .log
            .level
            .clone()
            .unwrap_or_else(|| "info".to_string());

        // Build and run the form
        let (enable_logging, selected_level, enable_console) =
            Self::run_form(settings, &current_level)?;

        // Update settings
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
        // For log, we consider it configured if we can load the config.
        // The original implementation always verified.
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

/// Get the Log stage instance.
pub fn log_stage() -> &'static dyn WorkflowStage {
    &LogStage
}
