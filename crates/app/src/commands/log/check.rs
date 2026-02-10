//! 检查日志级别命令

use crate::interactive::core::stage::WorkflowExecutor;
use crate::interactive::platforms::log::log_stage;
use prompt::{br, separator};

/// Log Check 命令
pub struct LogCheckCommand;

impl Default for LogCheckCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl LogCheckCommand {
    /// 创建新的 LogCheckCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow log check` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        separator!('─', 80, "Log Configuration Check");
        br!();
        let executor = WorkflowExecutor::new(log_stage());
        executor.run_verify()?;

        Ok(())
    }
}
