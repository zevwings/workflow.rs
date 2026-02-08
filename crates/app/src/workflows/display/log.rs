//! Log 验证结果格式化实现

use crate::workflows::display::formatter::VerificationResultFormatter;
use domain::LogVerificationResult;
use prompt::{br, info};

impl VerificationResultFormatter for LogVerificationResult {
    fn format(&self) {
        if !self.configured {
            info!("日志级别: 未配置（日志已禁用）");
            br!();
            return;
        }

        if let Some(ref config) = self.config {
            let level = config.level.as_deref().unwrap_or("未配置");
            info!("日志级别: {}", level);

            let enable_trace_console = config.enable_trace_console;
            info!(
                "追踪控制台: {}",
                if enable_trace_console {
                    "已启用"
                } else {
                    "已禁用"
                }
            );
        }
    }
}
