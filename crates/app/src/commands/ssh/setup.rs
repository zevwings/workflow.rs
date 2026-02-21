//! SSH 设置命令

use crate::bootstrap;
use crate::interactive::{core::stage::WorkflowExecutor, SSH_STAGE_NAME};

/// SSH Setup 命令
pub struct SshSetupCommand;

impl Default for SshSetupCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl SshSetupCommand {
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow ssh setup` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stage = bootstrap::get_workflow_stage_registry()
            .stage_by_name(SSH_STAGE_NAME)
            .expect("SSH stage must be registered");
        WorkflowExecutor::new(stage).run_command_setup()
    }
}
