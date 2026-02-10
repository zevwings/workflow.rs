//! 检查 LLM 配置命令

use crate::interactive::core::stage::WorkflowExecutor;
use crate::interactive::platforms::llm::llm_stage;
use prompt::{br, separator};

/// Llm Check 命令
pub struct LlmCheckCommand;

impl Default for LlmCheckCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmCheckCommand {
    /// 创建新的 LlmCheckCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow llm check` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        separator!('─', 80, "LLM Configuration Check");
        br!();
        WorkflowExecutor::new(llm_stage()).run_verify()
    }
}
