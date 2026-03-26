//! 检查 Codeup 配置命令

use prompt::{br, separator};

use crate::bootstrap;
use crate::interactive::{WorkflowExecutor, CODEUP_STAGE_NAME};

/// Codeup Check 命令
pub struct CodeupCheckCommand;

impl Default for CodeupCheckCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeupCheckCommand {
    /// 创建新的 CodeupCheckCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow codeup check` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        separator!('─', 80, "Codeup Configuration Check");
        br!();
        let stage = bootstrap::get_workflow_stage_registry()
            .stage_by_name(CODEUP_STAGE_NAME)
            .expect("Codeup stage must be registered");
        WorkflowExecutor::new(stage).run_verify()
    }
}
