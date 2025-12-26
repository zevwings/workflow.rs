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
        let pull_request_id = resolve_pull_request_id(pull_request_id)?;

        log_break!();
        log_success!("Closing PR: #{}", pull_request_id);

        let current_branch = GitBranch::current_branch()?;
        let pr_branch = Self::get_pr_branch_name(&pull_request_id)?;

        let default_branch = GitBranch::get_default_branch()?;

        if pr_branch == default_branch {
            color_eyre::eyre::bail!(
                "Cannot close PR on default branch '{}'. The PR branch should be a feature branch.",
                default_branch
            );
        }

        let was_already_closed = Self::check_if_already_closed(&pull_request_id)?;

        if !was_already_closed {
            // 如果关闭失败，检查是否是"已关闭"错误（竞态条件）
            if let Err(e) = Self::close_pull_request(&pull_request_id) {
                if helpers::is_pr_already_closed_error(&e) {
                    log_warning!(
                        "PR #{} has already been closed (detected from close error)",
                        pull_request_id
                    );
                    log_info!("Skipping close step, continuing with cleanup...");
                } else {
                    return Err(e);
                }
            }
        }

        Self::delete_remote_branch(&pr_branch)?;

        Self::cleanup_after_close(&current_branch, &pr_branch, &default_branch)?;

        Ok(())
    }

    /// 检查 PR 是否已经关闭
    fn check_if_already_closed(pull_request_id: &str) -> Result<bool> {
        let provider = create_provider_auto()?;
        let status = provider.get_pull_request_status(pull_request_id)?;

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

        match GitBranch::delete_remote(branch_name) {
            Ok(()) => {
                log_success!("Remote branch deleted: {}", branch_name);
            }
            Err(e) => {
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
                    log_warning!("Failed to delete remote branch: {}", e);
                    log_warning!("You may need to delete it manually");
                }
            }
        }

        Ok(())
    }

    /// 获取 PR 对应的分支名
    fn get_pr_branch_name(pull_request_id: &str) -> Result<String> {
        let provider = create_provider_auto()?;

        let info = provider.get_pull_request_info(pull_request_id)?;

        // PR 信息格式包含 "Source Branch: branch_name"
        for line in info.lines() {
            if let Some(branch_line) = line.strip_prefix("Source Branch: ") {
                return Ok(branch_line.trim().to_string());
            }
        }

        color_eyre::eyre::bail!("Failed to extract branch name from PR #{}", pull_request_id)
    }

    /// 关闭后清理：智能判断是否需要切换分支，并删除 PR 分支
    fn cleanup_after_close(
        current_branch: &str,
        pr_branch: &str,
        default_branch: &str,
    ) -> Result<()> {
        if current_branch == pr_branch {
            log_info!(
                "Currently on PR branch '{}', switching to default branch '{}' before deletion",
                pr_branch,
                default_branch
            );
            helpers::cleanup_branch(current_branch, default_branch, "PR close")
        } else {
            log_info!(
                "Currently on '{}', will delete PR branch '{}' without switching",
                current_branch,
                pr_branch
            );
            Self::delete_pr_branch_only(pr_branch)
        }
    }

    /// 仅删除指定的 PR 分支（不切换当前分支）
    fn delete_pr_branch_only(pr_branch: &str) -> Result<()> {
        use crate::git::GitRepo;

        GitRepo::fetch()?;

        if GitBranch::has_local_branch(pr_branch)? {
            log_info!("Deleting local PR branch: {}", pr_branch);
            GitBranch::delete(pr_branch, false)
                .or_else(|_| {
                    log_info!("Branch may not be fully merged, trying force delete...");
                    GitBranch::delete(pr_branch, true)
                })
                .wrap_err("Failed to delete local PR branch")?;
            log_success!("Local PR branch deleted: {}", pr_branch);
        } else {
            log_info!("Local PR branch already deleted: {}", pr_branch);
        }

        if let Err(e) = GitRepo::prune_remote() {
            log_info!("Warning: Failed to prune remote references: {}", e);
            log_info!("This is a non-critical cleanup operation. Local cleanup is complete.");
        }

        log_success!("PR branch '{}' cleanup completed", pr_branch);
        Ok(())
    }
}
