//! 设置日志级别命令

use crate::bootstrap;
use crate::interactive::{core::stage::WorkflowExecutor, LOG_STAGE_NAME};

/// Log Setup 命令
pub struct LogSetupCommand;

impl Default for LogSetupCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl LogSetupCommand {
    /// 创建新的 LogSetupCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow log setup` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stage = bootstrap::get_workflow_stage_registry()
            .stage_by_name(LOG_STAGE_NAME)
            .expect("Log stage must be registered");
        WorkflowExecutor::new(stage).run_command_setup()
    }
}
