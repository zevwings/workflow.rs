use crate::commands::pr::helpers;
use crate::git::GitBranch;
use crate::pr::create_provider_auto;
use crate::pr::helpers::resolve_pull_request_id;
use crate::{log_break, log_info, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};

/// PR 关闭命令
#[allow(dead_code)]
pub struct PullRequestCloseCommand;

#[allow(dead_code)]
impl PullRequestCloseCommand {
    /// 关闭 PR
    pub fn close(pull_request_id: Option<String>) -> Result<()> {
        // 1. 获取 PR ID
        let pull_request_id = resolve_pull_request_id(pull_request_id)?;

        log_break!();
        log_success!("Closing PR: #{}", pull_request_id);

        // 2. 获取当前分支名（关闭前保存）
        let current_branch = GitBranch::current_branch()?;

        // 3. 获取默认分支
        let default_branch = GitBranch::get_default_branch()?;

        // 4. 提前检查：如果当前分支是默认分支，不应该关闭
        if current_branch == default_branch {
            color_eyre::eyre::bail!(
                "Cannot close PR on default branch '{}'. Please switch to a feature branch first.",
                default_branch
            );
        }

        // 5. 检查 PR 状态（如果已关闭，跳过关闭步骤）
        let was_already_closed = Self::check_if_already_closed(&pull_request_id)?;

        if !was_already_closed {
            // 6. 关闭 PR（远程）
            // 如果关闭失败，检查是否是"已关闭"错误（竞态条件）
            if let Err(e) = Self::close_pull_request(&pull_request_id) {
                if helpers::is_pr_already_closed_error(&e) {
                    log_warning!(
                        "PR #{} has already been closed (detected from close error)",
                        pull_request_id
                    );
                    log_info!("Skipping close step, continuing with cleanup...");
                } else {
                    // 其他错误，返回错误
                    return Err(e);
                }
            }
        }

        // 7. 删除远程分支
        Self::delete_remote_branch(&current_branch)?;

        // 8. 清理本地：切换到默认分支并删除当前分支
        Self::cleanup_after_close(&current_branch, &default_branch)?;

        Ok(())
    }

    /// 检查 PR 是否已经关闭
    fn check_if_already_closed(pull_request_id: &str) -> Result<bool> {
        let provider = create_provider_auto()?;
        let status = provider.get_pull_request_status(pull_request_id)?;

        // 如果状态是 closed 或 merged，说明已经关闭
        if status.state == "closed" || status.state == "merged" {
            log_warning!(
                "PR #{} is already closed (state: {})",
                pull_request_id,
                status.state
            );
            return Ok(true);
        }

        Ok(false)
    }

    /// 关闭 PR（根据仓库类型调用对应的实现）
    fn close_pull_request(pull_request_id: &str) -> Result<()> {
        let provider = create_provider_auto()?;
        provider.close_pull_request(pull_request_id).wrap_err("Failed to close PR")?;
        log_success!("PR closed successfully");
        Ok(())
    }

    /// 删除远程分支
    fn delete_remote_branch(branch_name: &str) -> Result<()> {
        // 检查远程分支是否存在
        let exists_remote = GitBranch::has_remote_branch(branch_name)
            .wrap_err("Failed to check if remote branch exists")?;

        if !exists_remote {
            log_info!(
                "Remote branch '{}' does not exist, skipping deletion",
                branch_name
            );
            return Ok(());
        }

        log_info!("Deleting remote branch: {}", branch_name);
        log_info!("Note: This will permanently delete the remote branch");

        // 尝试删除远程分支
        match GitBranch::delete_remote(branch_name) {
            Ok(()) => {
                log_success!("Remote branch deleted: {}", branch_name);
            }
            Err(e) => {
                // 如果分支已经被 API 删除，忽略错误
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("remote ref does not exist")
                    || error_msg.contains("not found")
                    || error_msg.contains("does not exist")
                {
                    log_info!(
                        "Remote branch '{}' may have already been deleted",
                        branch_name
                    );
                } else {
                    // 其他错误，记录警告但继续执行
                    log_warning!("Failed to delete remote branch: {}", e);
                    log_warning!("You may need to delete it manually");
                }
            }
        }

        Ok(())
    }

    /// 关闭后清理：切换到默认分支并删除当前分支
    fn cleanup_after_close(current_branch: &str, default_branch: &str) -> Result<()> {
        helpers::cleanup_branch(current_branch, default_branch, "PR close")
    }
}
