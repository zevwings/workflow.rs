//! 设置 LLM 配置命令

use crate::bootstrap;
use crate::interactive::{core::stage::WorkflowExecutor, LLM_STAGE_NAME};

/// Llm Setup 命令
pub struct LlmSetupCommand;

impl Default for LlmSetupCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmSetupCommand {
    /// 创建新的 LlmSetupCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow llm setup` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stage = bootstrap::get_workflow_stage_registry()
            .stage_by_name(LLM_STAGE_NAME)
            .expect("LLM stage must be registered");
        WorkflowExecutor::new(stage).run_command_setup()
    }
}
