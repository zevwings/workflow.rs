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

        // 2. 获取当前分支名和 PR 对应的分支名
        let current_branch = GitBranch::current_branch()?;
        let pr_branch = Self::get_pr_branch_name(&pull_request_id)?;

        // 3. 获取默认分支
        let default_branch = GitBranch::get_default_branch()?;

        // 4. 提前检查：如果 PR 分支是默认分支，不应该关闭
        if pr_branch == default_branch {
            color_eyre::eyre::bail!(
                "Cannot close PR on default branch '{}'. The PR branch should be a feature branch.",
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
        Self::delete_remote_branch(&pr_branch)?;

        // 8. 智能清理：根据当前分支和 PR 分支的关系决定是否需要切换
        Self::cleanup_after_close(&current_branch, &pr_branch, &default_branch)?;

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

    /// 获取 PR 对应的分支名
    fn get_pr_branch_name(pull_request_id: &str) -> Result<String> {
        let provider = create_provider_auto()?;

        // 从 PR 信息中获取源分支名
        let info = provider.get_pull_request_info(pull_request_id)?;

        // 解析 PR 信息，提取源分支名
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
        // 情况判断：
        // 1. 如果当前分支就是要删除的 PR 分支，需要切换到默认分支
        // 2. 如果当前分支不是要删除的 PR 分支，只删除 PR 分支，不切换

        if current_branch == pr_branch {
            // 情况二：在要删除的分支上，需要切换到默认分支
            log_info!(
                "Currently on PR branch '{}', switching to default branch '{}' before deletion",
                pr_branch,
                default_branch
            );
            helpers::cleanup_branch(current_branch, default_branch, "PR close")
        } else {
            // 情况一：不在要删除的分支上，只删除 PR 分支，不切换当前分支
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

        // 1. 更新远程分支信息
        GitRepo::fetch()?;

        // 2. 删除本地分支（如果存在）
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

        // 3. 清理远程分支引用
        if let Err(e) = GitRepo::prune_remote() {
            log_info!("Warning: Failed to prune remote references: {}", e);
            log_info!("This is a non-critical cleanup operation. Local cleanup is complete.");
        }

        log_success!("PR branch '{}' cleanup completed", pr_branch);
        Ok(())
    }
}
