use crate::commands::check;
use crate::commands::pr::helpers;
use crate::jira::status::JiraStatus;
use crate::{
    detect_repo_type, extract_jira_ticket_id, log_break, log_info, log_success, log_warning,
    Codeup, Git, GitHub, Jira, PlatformProvider, RepoType,
};
use anyhow::Result;

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
        let default_branch = Git::get_default_branch()?;

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
    fn merge_pull_request(pull_request_id: &str, _repo_type: &RepoType) -> Result<bool> {
        // 先检查 PR 状态
        let status = detect_repo_type(
            |repo_type| match repo_type {
                RepoType::GitHub => GitHub::get_pull_request_status(pull_request_id),
                RepoType::Codeup => Codeup::get_pull_request_status(pull_request_id),
                RepoType::Unknown => {
                    anyhow::bail!(
                        "PR merge is currently only supported for GitHub and Codeup repositories."
                    );
                }
            },
            "get pull request status",
        )?;

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
        detect_repo_type(
            |repo_type| match repo_type {
                RepoType::GitHub => {
                    match GitHub::merge_pull_request(pull_request_id, true) {
                        Ok(()) => {
                            log_success!("PR merged successfully");
                            Ok(true)
                        }
                        Err(e) => {
                            // 检查是否是"已合并"错误
                            if helpers::is_pr_already_merged_error(&e) {
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
                    match Codeup::merge_pull_request(pull_request_id, true) {
                        Ok(()) => {
                            log_success!("PR merged successfully");
                            Ok(true)
                        }
                        Err(e) => {
                            // 检查是否是"已合并"错误
                            if helpers::is_pr_already_merged_error(&e) {
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
                RepoType::Unknown => {
                    anyhow::bail!(
                        "PR merge is currently only supported for GitHub and Codeup repositories."
                    );
                }
            },
            "merge pull request",
        )
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
        _repo_type: &RepoType,
    ) -> Result<Option<String>> {
        let title = detect_repo_type(
            |repo_type| match repo_type {
                RepoType::GitHub => GitHub::get_pull_request_title(pull_request_id),
                RepoType::Codeup => Codeup::get_pull_request_title(pull_request_id),
                RepoType::Unknown => Ok("".to_string()),
            },
            "get pull request title",
        )
        .ok();

        Ok(title.and_then(|t| extract_jira_ticket_id(&t)))
    }

    /// 合并后清理：切换到默认分支并删除当前分支
    fn cleanup_after_merge(current_branch: &str, default_branch: &str) -> Result<()> {
        log_info!(
            "Note: Remote branch '{}' may have already been deleted via API",
            current_branch
        );
        helpers::cleanup_branch(current_branch, default_branch, "PR merge")?;
        log_info!(
            "Note: Remote branch '{}' may have already been deleted via API",
            current_branch
        );
        Ok(())
    }
}
