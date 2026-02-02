//! 设置 LLM 配置命令

use crate::workflows::core::stage::WorkflowExecutor;
use crate::workflows::platforms::llm::llm_stage;

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
        WorkflowExecutor::new(llm_stage()).run_command_setup()
    }
}
