//! 关闭 Pull Request 命令

use prompt::{spinner, success};

use crate::registry;

/// Pull Request Close 命令
pub struct PullRequestCloseCommand {
    pr_id: String,
}

impl PullRequestCloseCommand {
    /// 创建新的 PullRequestCloseCommand
    pub fn new(pr_id: String) -> Self {
        Self { pr_id }
    }

    /// 运行 `workflow pr close` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pr_service = registry::get_pull_request_service();

        // 关闭 PR
        spinner!("Closing PR #{}...", self.pr_id)
            .with(|| pr_service.close_pull_request(&self.pr_id))
            .map_err(|e| format!("Failed to close Pull Request: {}", e))?;

        success!("Pull Request #{} closed successfully!", self.pr_id);

        Ok(())
    }
}
