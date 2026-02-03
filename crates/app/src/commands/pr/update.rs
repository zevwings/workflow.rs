//! 更新 Pull Request 命令

use crate::registry;
use color_eyre::Result;
use prompt::{error, spinner, success};

/// Pull Request Update 命令
pub struct PullRequestUpdateCommand {
    pr_id: String,
    title: Option<String>,
    body: Option<String>,
}

impl PullRequestUpdateCommand {
    /// 创建新的 PullRequestUpdateCommand
    pub fn new(pr_id: String, title: Option<String>, body: Option<String>) -> Self {
        Self { pr_id, title, body }
    }

    /// 运行 `workflow pr update` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pr_service = registry::get_pull_request_service();

        // 如果没有提供 title 和 body，提示用户至少提供一个
        if self.title.is_none() && self.body.is_none() {
            error!("At least one of title or body must be provided");
            return Err("At least one of title or body must be provided".into());
        }

        // 更新 PR
        spinner!("Updating PR #{}...", self.pr_id)
            .with(|| {
                pr_service.update_pull_request(
                    &self.pr_id,
                    self.title.as_deref(),
                    self.body.as_deref(),
                )
            })
            .map_err(|e| format!("Failed to update Pull Request: {}", e))?;

        success!("Pull Request #{} updated successfully!", self.pr_id);

        Ok(())
    }
}
