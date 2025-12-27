use crate::log_success;
use crate::pr::create_provider_auto;
use crate::pr::helpers::resolve_pull_request_id;
use color_eyre::{eyre::WrapErr, Result};

/// PR 评论命令
pub struct PullRequestCommentCommand;

impl PullRequestCommentCommand {
    /// 添加评论到 Pull Request
    pub fn comment(pull_request_id: Option<String>, message: Vec<String>) -> Result<()> {
        // 获取评论内容（将多个单词组合成一个字符串）
        if message.is_empty() {
            color_eyre::eyre::bail!("Comment message is required. Please provide a message.");
        }
        let comment_message = message.join(" ");

        // 获取 PR ID（从参数或当前分支）
        let pr_id = resolve_pull_request_id(pull_request_id)?;

        log_success!("Adding comment to PR: #{}", pr_id);

        // 创建平台提供者并添加评论
        let provider = create_provider_auto()?;
        provider
            .add_comment(&pr_id, &comment_message)
            .wrap_err(format!("Failed to add comment to PR #{}", pr_id))?;

        log_success!("Comment added to PR #{} successfully!", pr_id);

        Ok(())
    }
}
