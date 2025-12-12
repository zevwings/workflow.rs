use crate::log_success;
use crate::pr::create_provider_auto;
use crate::pr::helpers::resolve_pull_request_id;
use anyhow::{Context, Result};

/// PR 评论命令
#[allow(dead_code)]
pub struct PullRequestCommentCommand;

#[allow(dead_code)]
impl PullRequestCommentCommand {
    /// 添加评论到 Pull Request
    pub fn comment(pull_request_id: Option<String>, message: Vec<String>) -> Result<()> {
        // 获取评论内容（将多个单词组合成一个字符串）
        if message.is_empty() {
            anyhow::bail!("Comment message is required. Please provide a message.");
        }
        let comment_message = message.join(" ");

        // 获取 PR ID（从参数或当前分支）
        let pr_id = resolve_pull_request_id(pull_request_id)?;

        log_success!("Adding comment to PR: #{}", pr_id);

        // 创建平台提供者并添加评论
        let provider = create_provider_auto()?;
        provider
            .add_comment(&pr_id, &comment_message)
            .context(format!("Failed to add comment to PR #{}", pr_id))?;

        log_success!("Comment added to PR #{} successfully!", pr_id);

        Ok(())
    }
}
