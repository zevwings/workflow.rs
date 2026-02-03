//! 批准 Pull Request 命令

use prompt::{spinner, success};

use crate::registry;

/// Pull Request Approve 命令
pub struct PullRequestApproveCommand {
    pr_id: String,
}

impl PullRequestApproveCommand {
    /// 创建新的 PullRequestApproveCommand
    pub fn new(pr_id: String) -> Self {
        Self { pr_id }
    }

    /// 运行 `workflow pr approve` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pr_service = registry::get_pull_request_service();

        // 批准 PR
        spinner!("Approving PR #{}...", self.pr_id)
            .with(|| pr_service.approve_pull_request(&self.pr_id))
            .map_err(|e| format!("Failed to approve Pull Request: {}", e))?;

        success!("Pull Request #{} approved successfully!", self.pr_id);

        Ok(())
    }
}
