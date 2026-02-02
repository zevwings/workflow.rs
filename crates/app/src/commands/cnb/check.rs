//! 检查 CNB 账号命令

use crate::workflows::core::stage::WorkflowExecutor;
use crate::workflows::platforms::cnb::cnb_stage;
use prompt::{br, separator};

/// CNB Check 命令
pub struct CNBCheckCommand;

impl Default for CNBCheckCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl CNBCheckCommand {
    /// 创建新的 CNBCheckCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow cnb check` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        separator!('─', 80, "CNB Configuration Check");
        br!();
        WorkflowExecutor::new(cnb_stage()).run_verify()
    }
}
