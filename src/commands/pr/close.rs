use crate::commands::pr::helpers;
use crate::{
    log_break, log_info, log_success, log_warning, Codeup, Git, GitHub, PlatformProvider, RepoType,
};
use anyhow::{Context, Result};

/// PR 关闭命令
#[allow(dead_code)]
pub struct PullRequestCloseCommand;

#[allow(dead_code)]
impl PullRequestCloseCommand {
    /// 关闭 PR
    pub fn close(pull_request_id: Option<String>) -> Result<()> {
        // 1. 获取仓库类型和 PR ID
        let repo_type = Git::detect_repo_type()?;
        let pull_request_id = helpers::resolve_pull_request_id(pull_request_id, &repo_type)?;

        log_break!();
        log_success!("Closing PR: #{}", pull_request_id);

        // 2. 获取当前分支名（关闭前保存）
        let current_branch = Git::current_branch()?;

        // 3. 获取默认分支
        let default_branch = Git::get_default_branch().context("Failed to get default branch")?;

        // 4. 提前检查：如果当前分支是默认分支，不应该关闭
        if current_branch == default_branch {
            anyhow::bail!(
                "Cannot close PR on default branch '{}'. Please switch to a feature branch first.",
                default_branch
            );
        }

        // 5. 检查 PR 状态（如果已关闭，跳过关闭步骤）
        let was_already_closed = Self::check_if_already_closed(&pull_request_id, &repo_type)?;

        if !was_already_closed {
            // 6. 关闭 PR（远程）
            // 如果关闭失败，检查是否是"已关闭"错误（竞态条件）
            if let Err(e) = Self::close_pull_request(&pull_request_id, &repo_type) {
                if Self::is_already_closed_error(&e) {
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
    fn check_if_already_closed(pull_request_id: &str, repo_type: &RepoType) -> Result<bool> {
        let status = match repo_type {
            RepoType::GitHub => {
                <GitHub as PlatformProvider>::get_pull_request_status(pull_request_id)?
            }
            RepoType::Codeup => {
                <Codeup as PlatformProvider>::get_pull_request_status(pull_request_id)?
            }
            _ => {
                anyhow::bail!(
                    "PR close is currently only supported for GitHub and Codeup repositories."
                );
            }
        };

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
    fn close_pull_request(pull_request_id: &str, repo_type: &RepoType) -> Result<()> {
        match repo_type {
            RepoType::GitHub => {
                <GitHub as PlatformProvider>::close_pull_request(pull_request_id)
                    .context("Failed to close PR via GitHub API")?;
                log_success!("PR closed successfully");
            }
            RepoType::Codeup => {
                <Codeup as PlatformProvider>::close_pull_request(pull_request_id)
                    .context("Failed to close PR via Codeup API")?;
                log_success!("PR closed successfully");
            }
            _ => {
                anyhow::bail!(
                    "PR close is currently only supported for GitHub and Codeup repositories."
                );
            }
        }
        Ok(())
    }

    /// 检查错误是否是"PR 已关闭"错误
    ///
    /// 这是一个备用检查，用于处理以下情况：
    /// 1. 状态检查失败（网络问题等）
    /// 2. 竞态条件：在状态检查和实际关闭之间，PR 被其他进程关闭了
    fn is_already_closed_error(error: &anyhow::Error) -> bool {
        let error_msg = error.to_string().to_lowercase();

        // 优先检查明确的错误消息
        if error_msg.contains("already been closed")
            || error_msg.contains("pull request has already been closed")
            || error_msg.contains("is already closed")
            || error_msg.contains("state is closed")
        {
            return true;
        }

        // 检查 HTTP 状态码（需要结合错误消息，避免误判）
        // 422 (Unprocessable Entity) - GitHub API 在 PR 已关闭时可能返回此状态码
        // 但需要确保错误消息中包含 close 相关的内容，避免误判其他错误
        if error_msg.contains("422")
            && (error_msg.contains("close") || error_msg.contains("closed"))
        {
            return true;
        }

        false
    }

    /// 删除远程分支
    fn delete_remote_branch(branch_name: &str) -> Result<()> {
        // 检查远程分支是否存在
        let (_, exists_remote) = Git::is_branch_exists(branch_name)
            .context("Failed to check if remote branch exists")?;

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
        match Git::delete_remote(branch_name) {
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
        // 如果当前分支已经是默认分支，不需要清理
        if current_branch == default_branch {
            log_info!("Already on default branch: {}", default_branch);
            return Ok(());
        }

        log_info!("Switching to default branch: {}", default_branch);

        // 1. 先更新远程分支信息（这会同步远程删除的分支）
        Git::fetch()?;

        // 2. 检查是否有未提交的更改，如果有就先 stash
        let has_stashed = Git::has_commit()?;
        if has_stashed {
            log_info!("Stashing local changes before switching branches...");
            Git::stash_push(Some("Auto-stash before PR close cleanup"))?;
        }

        // 3. 切换到默认分支
        Git::checkout_branch(default_branch)
            .with_context(|| format!("Failed to checkout default branch: {}", default_branch))?;

        // 4. 更新本地默认分支
        Git::pull(default_branch)
            .with_context(|| format!("Failed to pull latest changes from {}", default_branch))?;

        // 5. 删除本地分支（如果还存在）
        if Self::branch_exists_locally(current_branch)? {
            log_info!("Deleting local branch: {}", current_branch);
            Git::delete(current_branch, false)
                .or_else(|_| {
                    // 如果分支未完全合并，尝试强制删除
                    log_info!("Branch may not be fully merged, trying force delete...");
                    Git::delete(current_branch, true)
                })
                .context("Failed to delete local branch")?;
            log_success!("Local branch deleted: {}", current_branch);
        } else {
            log_info!("Local branch already deleted: {}", current_branch);
        }

        // 6. 恢复 stash（如果有）
        if has_stashed {
            log_info!("Restoring stashed changes...");
            if let Err(e) = Git::stash_pop() {
                log_warning!("Failed to restore stashed changes: {}", e);
                log_warning!("You can manually restore them with: git stash pop");
                log_warning!("Or view the stash list with: git stash list");
            } else {
                log_success!("Stashed changes restored successfully");
            }
        }

        // 7. 清理远程分支引用（prune，移除已删除的远程分支引用）
        if let Err(e) = Git::prune_remote() {
            log_info!("Warning: Failed to prune remote references: {}", e);
            log_info!("This is a non-critical cleanup operation. Local cleanup is complete.");
        }

        log_success!(
            "Cleanup completed: switched to {} and deleted local branch {}",
            default_branch,
            current_branch
        );

        Ok(())
    }

    /// 检查本地分支是否存在
    fn branch_exists_locally(branch_name: &str) -> Result<bool> {
        let (exists_local, _) =
            Git::is_branch_exists(branch_name).context("Failed to check if branch exists")?;
        Ok(exists_local)
    }
}
