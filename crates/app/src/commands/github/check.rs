//! 检查 GitHub 账号命令

use crate::workflows::core::stage::WorkflowExecutor;
use crate::workflows::platforms::github::github_stage;
use prompt::{br, separator};

/// Github Check 命令
pub struct GithubCheckCommand;

impl Default for GithubCheckCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl GithubCheckCommand {
    /// 创建新的 GithubCheckCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow github check` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        separator!('─', 80, "GitHub Configuration Check");
        br!();
        WorkflowExecutor::new(github_stage()).run_verify()
    }
}
