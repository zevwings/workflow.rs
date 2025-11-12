use crate::commands::check;
use crate::commands::pr::helpers;
use crate::jira::status::JiraStatus;
use crate::{
    extract_jira_ticket_id, log_break, log_info, log_success, log_warning, Codeup, Git, GitHub,
    Jira, PlatformProvider, RepoType,
};
use anyhow::{Context, Result};

/// PR 合并命令
#[allow(dead_code)]
pub struct PullRequestMergeCommand;

#[allow(dead_code)]
impl PullRequestMergeCommand {
    /// 合并 PR
    pub fn merge(pull_request_id: Option<String>, _force: bool) -> Result<()> {
        // 1. 运行检查
        check::run_all()?;

        // 2. 获取仓库类型和 PR ID
        let repo_type = Git::detect_repo_type()?;
        let pull_request_id = helpers::resolve_pull_request_id(pull_request_id, &repo_type)?;

        log_break!();
        log_success!("Merging PR: #{}", pull_request_id);

        // 3. 获取当前分支名（合并前保存）
        let current_branch = Git::current_branch()?;

        // 4. 获取默认分支
        let default_branch = Git::get_default_branch().context("Failed to get default branch")?;

        // 5. 合并 PR（如果已合并，跳过合并步骤但继续执行后续步骤）
        Self::merge_pull_request(&pull_request_id, &repo_type)?;

        // 6. 合并后清理：切换到默认分支并删除当前分支
        // 注意：如果 PR 已合并，远程分支可能已经被删除
        Self::cleanup_after_merge(&current_branch, &default_branch)?;

        // 7. 更新 Jira 状态（如果关联了 ticket）
        Self::update_jira_status(&pull_request_id, &repo_type)?;

        Ok(())
    }

    /// 合并 PR（根据仓库类型调用对应的实现）
    /// 返回 true 表示新合并，false 表示已经合并
    fn merge_pull_request(pull_request_id: &str, repo_type: &RepoType) -> Result<bool> {
        // 先检查 PR 状态
        let status = match repo_type {
            RepoType::GitHub => {
                <GitHub as PlatformProvider>::get_pull_request_status(pull_request_id)?
            }
            RepoType::Codeup => {
                <Codeup as PlatformProvider>::get_pull_request_status(pull_request_id)?
            }
            _ => {
                anyhow::bail!(
                    "PR merge is currently only supported for GitHub and Codeup repositories."
                );
            }
        };

        // 如果已经合并，跳过合并步骤
        if status.merged {
            log_warning!("PR #{} has already been merged", pull_request_id);
            if let Some(merged_at) = status.merged_at {
                log_info!("Merged at: {}", merged_at);
            }
            log_info!("Skipping merge step, continuing with cleanup...");
            return Ok(false);
        }

        // 执行合并操作
        match repo_type {
            RepoType::GitHub => {
                match <GitHub as PlatformProvider>::merge_pull_request(pull_request_id, true) {
                    Ok(()) => {
                        log_success!("PR merged successfully");
                        Ok(true)
                    }
                    Err(e) => {
                        // 检查是否是"已合并"错误
                        if Self::is_already_merged_error(&e) {
                            log_warning!(
                                "PR #{} has already been merged (detected from merge error)",
                                pull_request_id
                            );
                            log_info!("Skipping merge step, continuing with cleanup...");
                            Ok(false)
                        } else {
                            // 其他错误，返回错误
                            Err(e)
                        }
                    }
                }
            }
            RepoType::Codeup => {
                match <Codeup as PlatformProvider>::merge_pull_request(pull_request_id, true) {
                    Ok(()) => {
                        log_success!("PR merged successfully");
                        Ok(true)
                    }
                    Err(e) => {
                        // 检查是否是"已合并"错误
                        if Self::is_already_merged_error(&e) {
                            log_warning!(
                                "PR #{} has already been merged (detected from merge error)",
                                pull_request_id
                            );
                            log_info!("Skipping merge step, continuing with cleanup...");
                            Ok(false)
                        } else {
                            // 其他错误，返回错误
                            Err(e)
                        }
                    }
                }
            }
            _ => {
                anyhow::bail!(
                    "PR merge is currently only supported for GitHub and Codeup repositories."
                );
            }
        }
    }

    /// 更新 Jira 状态（如果关联了 ticket）
    fn update_jira_status(pull_request_id: &str, repo_type: &RepoType) -> Result<()> {
        // 获取当前仓库 URL
        let repository = Git::get_remote_url().ok();

        // 尝试从历史记录读取
        let mut jira_ticket =
            JiraStatus::read_work_history(pull_request_id, repository.as_deref())?;

        // 如果历史记录中没有，尝试从 PR 标题提取
        if jira_ticket.is_none() {
            jira_ticket = Self::extract_jira_ticket_from_pr_title(pull_request_id, repo_type)?;
        }

        if let Some(ticket) = jira_ticket {
            // 读取合并时的状态
            if let Ok(Some(status)) = JiraStatus::read_pull_request_merged_status(&ticket) {
                log_success!("Updating Jira ticket: {} to status: {}", ticket, status);
                Jira::move_ticket(&ticket, &status)?;
                log_success!("Jira ticket updated");
            } else {
                log_warning!("No Jira status configuration found for ticket: {}", ticket);
            }
        } else {
            log_warning!("No Jira ticket associated with this PR");
        }

        // 删除工作历史记录中的 PR ID 条目
        JiraStatus::delete_work_history_entry(pull_request_id, repository.as_deref())?;

        Ok(())
    }

    /// 从 PR 标题提取 Jira ticket ID
    fn extract_jira_ticket_from_pr_title(
        pull_request_id: &str,
        repo_type: &RepoType,
    ) -> Result<Option<String>> {
        let title = match repo_type {
            RepoType::GitHub => {
                <GitHub as PlatformProvider>::get_pull_request_title(pull_request_id).ok()
            }
            RepoType::Codeup => {
                <Codeup as PlatformProvider>::get_pull_request_title(pull_request_id).ok()
            }
            _ => None,
        };

        Ok(title.and_then(|t| extract_jira_ticket_id(&t)))
    }

    /// 合并后清理：切换到默认分支并删除当前分支
    fn cleanup_after_merge(current_branch: &str, default_branch: &str) -> Result<()> {
        // 如果当前分支已经是默认分支，不需要清理
        if current_branch == default_branch {
            log_info!("Already on default branch: {}", default_branch);
            return Ok(());
        }

        log_info!("Switching to default branch: {}", default_branch);
        log_info!(
            "Note: Remote branch '{}' may have already been deleted via API",
            current_branch
        );

        // 1. 先更新远程分支信息（这会同步远程删除的分支）
        Git::fetch()?;

        // 2. 检查是否有未提交的更改，如果有就先 stash
        let has_stashed = Git::has_commit()?;
        if has_stashed {
            log_info!("Stashing local changes before switching branches...");
            Git::stash_push(Some("Auto-stash before PR merge cleanup"))?;
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
        // 注意：这是一个可选的清理操作，失败不影响主要功能
        if let Err(e) = Git::prune_remote() {
            log_info!("Warning: Failed to prune remote references: {}", e);
            log_info!("This is a non-critical cleanup operation. Local cleanup is complete.");
        }

        log_success!(
            "Cleanup completed: switched to {} and deleted local branch {}",
            default_branch,
            current_branch
        );
        log_info!(
            "Note: Remote branch '{}' may have already been deleted via API",
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

    /// 检查错误是否是"PR 已合并"错误
    ///
    /// 这是一个备用检查，用于处理以下情况：
    /// 1. 状态检查失败（网络问题等）
    /// 2. 竞态条件：在状态检查和实际合并之间，PR 被其他进程合并了
    fn is_already_merged_error(error: &anyhow::Error) -> bool {
        let error_msg = error.to_string().to_lowercase();

        // 优先检查明确的错误消息
        if error_msg.contains("already been merged")
            || error_msg.contains("pull request has already been merged")
            || error_msg.contains("not mergeable")
        {
            return true;
        }

        // 检查 HTTP 状态码（需要结合错误消息，避免误判）
        // 405 (Method Not Allowed) - 某些 API 在 PR 已合并时返回此状态码
        // 422 (Unprocessable Entity) - GitHub API 在 PR 已合并时可能返回此状态码
        // 但需要确保错误消息中包含 merge 相关的内容，避免误判其他错误
        if error_msg.contains("405") && error_msg.contains("merge") {
            return true;
        }
        if error_msg.contains("422") && error_msg.contains("merge") {
            return true;
        }

        false
    }
}
