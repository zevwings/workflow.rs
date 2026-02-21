//! 设置 GitHub 账号命令

use crate::bootstrap;
use crate::interactive::{core::stage::WorkflowExecutor, GITHUB_STAGE_NAME};

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
        let stage = bootstrap::get_workflow_stage_registry()
            .stage_by_name(GITHUB_STAGE_NAME)
            .expect("GitHub stage must be registered");
        WorkflowExecutor::new(stage).run_command_setup()
    }
}
