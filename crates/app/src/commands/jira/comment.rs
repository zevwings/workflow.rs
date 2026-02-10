//! 向 Jira ticket 添加评论命令

use prompt::{spinner, success};

use crate::registry;
use crate::utils::get_jira_id_interactive;

/// Jira Comment 命令
pub struct JiraCommentCommand {
    jira_id: Option<String>,
    message: String,
}

impl JiraCommentCommand {
    /// 创建新的 JiraCommentCommand
    pub fn new(jira_id: Option<String>, message: String) -> Self {
        Self { jira_id, message }
    }

    /// 运行 `workflow jira comment` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let jira_id = get_jira_id_interactive(self.jira_id.clone())?;
        let jira_repo = registry::get_jira_repository();

        spinner!("Adding comment to Jira ticket {}...", jira_id).with(|| {
            jira_repo
                .add_comment(&jira_id, &self.message)
                .map_err(|e| format!("Failed to add comment: {}", e))
        })?;

        success!("Comment added to Jira ticket {}", jira_id);
        Ok(())
    }
}
