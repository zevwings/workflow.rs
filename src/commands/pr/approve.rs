use crate::pr::create_provider;
use crate::pr::helpers::resolve_pull_request_id;
use crate::{log_success, ProxyManager};
use anyhow::{Context, Result};

/// PR 批准命令
#[allow(dead_code)]
pub struct PullRequestApproveCommand;

#[allow(dead_code)]
impl PullRequestApproveCommand {
    /// 批准 Pull Request
    pub fn approve(pull_request_id: Option<String>) -> Result<()> {
        // 如果 VPN 开启，自动启用代理
        ProxyManager::ensure_proxy_enabled().context("Failed to enable proxy")?;

        // 获取 PR ID（从参数或当前分支）
        let pr_id = resolve_pull_request_id(pull_request_id)?;

        log_success!("Approving PR: #{}", pr_id);

        // 创建平台提供者并批准 PR
        let provider = create_provider()?;
        provider
            .approve_pull_request(&pr_id)
            .context(format!("Failed to approve PR #{}", pr_id))?;

        log_success!("PR #{} approved successfully!", pr_id);

        Ok(())
    }
}

