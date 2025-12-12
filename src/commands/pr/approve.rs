use crate::log_success;
use crate::pr::create_provider_auto;
use crate::pr::helpers::resolve_pull_request_id;
use anyhow::{Context, Result};

/// PR 批准命令
#[allow(dead_code)]
pub struct PullRequestApproveCommand;

#[allow(dead_code)]
impl PullRequestApproveCommand {
    /// 批准 Pull Request
    pub fn approve(pull_request_id: Option<String>) -> Result<()> {
        // 获取 PR ID（从参数或当前分支）
        let pr_id = resolve_pull_request_id(pull_request_id)?;

        log_success!("Approving PR: #{}", pr_id);

        // 创建平台提供者并批准 PR
        let provider = create_provider_auto()?;
        match provider.approve_pull_request(&pr_id) {
            Ok(_) => {
                log_success!("PR #{} approved successfully!", pr_id);
            }
            Err(e) => {
                // 检查是否是"不能批准自己的 PR"这种明确的业务错误
                let error_msg = e.to_string();
                if error_msg.contains("Cannot approve your own pull request") {
                    // 对于这种明确的业务错误，直接返回错误，不添加额外的上下文
                    return Err(e);
                } else {
                    // 对于其他错误，添加上下文信息
                    return Err(e).context(format!("Failed to approve PR #{}", pr_id));
                }
            }
        }

        Ok(())
    }
}
