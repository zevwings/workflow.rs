//! 设置日志级别命令

use crate::workflows::core::stage::WorkflowExecutor;
use crate::workflows::platforms::log::log_stage;

/// Log Setup 命令
pub struct LogSetupCommand;

impl Default for LogSetupCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl LogSetupCommand {
    /// 创建新的 LogSetupCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow log setup` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 使用 v2 WorkflowExecutor
        let executor = WorkflowExecutor::new(log_stage());
        executor.run_command_setup()?;
        Ok(())
    }
}
