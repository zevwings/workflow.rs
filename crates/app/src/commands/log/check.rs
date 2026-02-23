//! 检查日志级别命令

use prompt::{br, separator};

use crate::bootstrap;
use crate::interactive::{WorkflowExecutor, LOG_STAGE_NAME};

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
        let stage = bootstrap::get_workflow_stage_registry()
            .stage_by_name(LOG_STAGE_NAME)
            .expect("Log stage must be registered");
        WorkflowExecutor::new(stage).run_verify()?;

        Ok(())
    }
}
