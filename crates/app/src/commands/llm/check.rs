//! 检查 LLM 配置命令

use prompt::{br, separator};

use crate::bootstrap;
use crate::interactive::{core::stage::WorkflowExecutor, LLM_STAGE_NAME};

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
        let stage = bootstrap::get_workflow_stage_registry()
            .stage_by_name(LLM_STAGE_NAME)
            .expect("LLM stage must be registered");
        WorkflowExecutor::new(stage).run_verify()
    }
}
