use crate::pr::create_provider_auto;
use crate::{log_break, log_message};
use color_eyre::Result;

/// PR 状态命令
pub struct PullRequestStatusCommand;

impl PullRequestStatusCommand {
    /// 显示 PR 状态信息
    pub fn show(pull_request_id_or_branch: Option<String>) -> Result<()> {
        // 获取 PR ID 或标识符
        let pr_identifier = Self::get_pr_identifier(pull_request_id_or_branch)?;

        // 显示 PR 信息
        Self::show_pr_info(&pr_identifier)?;

        Ok(())
    }

    /// 获取 PR 标识符（从参数或当前分支）
    fn get_pr_identifier(pull_request_id_or_branch: Option<String>) -> Result<String> {
        if let Some(id) = pull_request_id_or_branch {
            // GitHub 只支持数字 ID
            // 尝试解析为数字 ID
            if id.parse::<u32>().is_ok() {
                Ok(id)
            } else {
                // 如果不是数字，返回原值（可能用于错误提示）
                Ok(id)
            }
        } else {
            // 从当前分支获取 PR
            crate::pr::helpers::resolve_pull_request_id(None)
        }
    }

    /// 显示 PR 信息
    fn show_pr_info(pr_identifier: &str) -> Result<()> {
        let provider = create_provider_auto()?;
        let info = provider.get_pull_request_info(pr_identifier)?;

        log_break!();
        log_break!('=', 40, "PR Information");
        log_message!("{}", info);
        Ok(())
    }
}
