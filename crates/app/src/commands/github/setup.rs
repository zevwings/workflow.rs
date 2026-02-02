//! 设置 GitHub 账号命令

use crate::workflows::core::stage::WorkflowExecutor;
use crate::workflows::platforms::github::github_stage;

/// Github Setup 命令
pub struct GithubSetupCommand;

impl Default for GithubSetupCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl GithubSetupCommand {
    /// 创建新的 GithubSetupCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow github setup` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        WorkflowExecutor::new(github_stage()).run_command_setup()
    }
}
