//! 检查 GitHub 账号命令

use prompt::{br, separator};

use crate::interactive::{core::stage::WorkflowExecutor, platforms::github::github_stage};

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
