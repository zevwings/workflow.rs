//! CNB 配置命令

use crate::workflows::core::stage::WorkflowExecutor;
use crate::workflows::platforms::cnb::cnb_stage;

/// CNB Setup 命令
pub struct CNBSetupCommand;

impl Default for CNBSetupCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl CNBSetupCommand {
    /// 创建新的 CNBSetupCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow cnb setup` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        WorkflowExecutor::new(cnb_stage()).run_command_setup()
    }
}
