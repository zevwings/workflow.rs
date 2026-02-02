//! 合并 Pull Request 命令

use crate::registry;
use color_eyre::Result;
use prompt::{info, success, Spinner};

/// Pull Request Merge 命令
pub struct PullRequestMergeCommand {
    pr_id: String,
    force: bool,
}

impl PullRequestMergeCommand {
    /// 创建新的 PullRequestMergeCommand
    pub fn new(pr_id: String, force: bool) -> Self {
        Self { pr_id, force }
    }

    /// 运行 `workflow pr merge` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pr_service = registry::get_pull_request_service();

        if self.force {
            info!("Force merging PR #{}...", self.pr_id);
        }

        // 合并 PR
        Spinner::new(format!("Merging PR #{}...", self.pr_id))
            .with(|| pr_service.merge_pull_request(&self.pr_id, self.force))
            .map_err(|e| format!("Failed to merge Pull Request: {}", e))?;

        success!("Pull Request #{} merged successfully!", self.pr_id);

        Ok(())
    }
}
