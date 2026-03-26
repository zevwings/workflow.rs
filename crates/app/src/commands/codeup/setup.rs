//! 设置 Codeup 配置命令

use crate::bootstrap;
use crate::interactive::{WorkflowExecutor, CODEUP_STAGE_NAME};

/// Codeup Setup 命令
pub struct CodeupSetupCommand;

impl Default for CodeupSetupCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeupSetupCommand {
    /// 创建新的 CodeupSetupCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow codeup setup` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stage = bootstrap::get_workflow_stage_registry()
            .stage_by_name(CODEUP_STAGE_NAME)
            .expect("Codeup stage must be registered");
        WorkflowExecutor::new(stage).run_command_setup()
    }
}
