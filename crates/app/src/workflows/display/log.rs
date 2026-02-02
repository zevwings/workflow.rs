//! Log 验证结果格式化实现

use crate::workflows::display::formatter::VerificationResultFormatter;
use domain::LogVerificationResult;
use prompt::{br, info};

impl VerificationResultFormatter for LogVerificationResult {
    fn format(&self) {
        if !self.configured {
            info!("Log level: Not configured (logging disabled)");
            br!();
            return;
        }

        if let Some(ref config) = self.config {
            let level = config.level.as_deref().unwrap_or("Not configured");
            info!("Log level: {}", level);

            let enable_trace_console = config.enable_trace_console;
            info!(
                "Trace console: {}",
                if enable_trace_console {
                    "Enabled"
                } else {
                    "Disabled"
                }
            );
        }
    }
}
