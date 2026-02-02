//! 添加 Pull Request 评论命令

use color_eyre::Result;
use prompt::{error, input, success, Spinner};

use crate::registry;

/// Pull Request Comment 命令
pub struct PullRequestCommentCommand {
    pr_id: String,
    comment: Option<String>,
}

impl PullRequestCommentCommand {
    /// 创建新的 PullRequestCommentCommand
    pub fn new(pr_id: String, comment: Option<String>) -> Self {
        Self { pr_id, comment }
    }

    /// 运行 `workflow pr comment` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pr_service = registry::get_pull_request_service();

        // 获取评论内容
        let comment = if let Some(c) = &self.comment {
            c.clone()
        } else {
            input!("Please enter your comment:")
                .prompt()
                .map_err(|e| format!("Failed to get comment: {}", e))?
        };

        if comment.is_empty() {
            error!("Comment cannot be empty");
            return Err("Comment cannot be empty".into());
        }

        // 添加评论
        Spinner::new(format!("Adding comment to PR #{}...", self.pr_id))
            .with(|| pr_service.add_comment(&self.pr_id, &comment))
            .map_err(|e| format!("Failed to add comment: {}", e))?;

        success!(
            "Comment added to Pull Request #{} successfully!",
            self.pr_id
        );

        Ok(())
    }
}
