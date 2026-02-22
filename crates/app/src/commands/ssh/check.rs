//! SSH 检查命令

use prompt::{br, separator};

use crate::bootstrap;
use crate::interactive::{core::stage::WorkflowExecutor, SSH_STAGE_NAME};

/// SSH Check 命令
pub struct SshCheckCommand;

impl Default for SshCheckCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl SshCheckCommand {
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow ssh check` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        separator!('─', 80, "SSH Configuration Check");
        br!();
        let stage = bootstrap::get_workflow_stage_registry()
            .stage_by_name(SSH_STAGE_NAME)
            .expect("SSH stage must be registered");
        WorkflowExecutor::new(stage).run_verify()?;

        Ok(())
    }
}
