use crate::pr::create_provider;
use crate::pr::helpers::resolve_pull_request_id;
use crate::{log_success, ProxyManager};
use anyhow::{Context, Result};

/// PR 评论命令
#[allow(dead_code)]
pub struct PullRequestCommentCommand;

#[allow(dead_code)]
impl PullRequestCommentCommand {
    /// 添加评论到 Pull Request
    pub fn comment(pull_request_id: Option<String>, message: String) -> Result<()> {
        // 如果 VPN 开启，自动启用代理
        ProxyManager::ensure_proxy_enabled().context("Failed to enable proxy")?;

        // 获取 PR ID（从参数或当前分支）
        let pr_id = resolve_pull_request_id(pull_request_id)?;

        log_success!("Adding comment to PR: #{}", pr_id);

        // 创建平台提供者并添加评论
        let provider = create_provider()?;
        provider
            .add_comment(&pr_id, &message)
            .context(format!("Failed to add comment to PR #{}", pr_id))?;

        log_success!("Comment added to PR #{} successfully!", pr_id);

        Ok(())
    }
}

