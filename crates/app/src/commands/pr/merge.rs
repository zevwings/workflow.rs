//! 合并 Pull Request 命令

use domain::extract_jira_ticket_id;
use prompt::{info, spinner, success, warning};
use toolkit::log_info;

use crate::{
    bootstrap::{
        get_git_repository, get_jira_repository, get_jira_work_history_repository,
        get_pull_request_service,
    },
    commands::pr::utils::get_pull_request_id_interactive_optional,
    util::{safe_pull, PullOptions},
};

/// Pull Request Merge 命令
pub struct PullRequestMergeCommand {
    pr_id: Option<String>,
    force: bool,
}

impl PullRequestMergeCommand {
    /// 创建新的 PullRequestMergeCommand
    pub fn new(pr_id: Option<String>, force: bool) -> Self {
        Self { pr_id, force }
    }

    /// 运行 `workflow pr merge` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pr_service = get_pull_request_service();
        let git_repo = get_git_repository();

        if self.force {
            info!("Force mode enabled: remote branch will be deleted after merge");
        }

        let pr_id_option = get_pull_request_id_interactive_optional(self.pr_id.clone())?;
        let pr_id = if let Some(pr_id) = pr_id_option {
            pr_id
        } else {
            let current_branch = git_repo.get_current_branch()?;
            let pr_id =
                spinner!("Searching for PR ID for branch '{}'...", current_branch).with(|| {
                    pr_service.get_current_branch_pull_request(&current_branch)?.ok_or_else(
                        || -> Box<dyn std::error::Error> {
                            "No PR found for current branch".into()
                        },
                    )
                })?;

            log_info!("Found PR ID: {}", pr_id);

            pr_id
        };

        // 1. 获取 PR 信息（在合并前获取源分支、目标分支和标题）
        let pr_info = spinner!("Fetching PR information...")
            .with(|| pr_service.get_pull_request(&pr_id))
            .map_err(|e| format!("Failed to get PR info: {}", e))?;
        let source_branch = pr_info.source_branch;
        let target_branch = pr_info.target_branch;
        let pr_title = pr_info.title.clone();

        // 2. 合并 PR
        spinner!("Merging PR #{}...", pr_id)
            .with(|| pr_service.merge_pull_request(&pr_id, self.force))
            .map_err(|e| format!("Failed to merge Pull Request: {}", e))?;

        success!("Pull Request #{} merged successfully!", pr_id);

        // 获取仓库 URL
        let repo_info = git_repo.get_repo_info();
        let repository_url = repo_info.origin_url.as_deref();

        // 3. 更新 Jira 状态
        self.update_jira_after_pr_merged(&pr_id, Some(pr_title.as_str()), repository_url)?;

        // 4. 切换到 target_branch
        let current_branch = git_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        let needs_stash = if current_branch != target_branch {
            spinner!("Switching to branch '{}'...", target_branch)
                .with(|| {
                    // 检查是否有未提交的更改
                    let status = git_repo
                        .get_working_tree_status()
                        .map_err(|e| format!("Failed to get status: {}", e))?;

                    let needs_stash = !status.is_clean();

                    if needs_stash {
                        log_info!("Stashing uncommitted changes before switching branch");
                        git_repo
                            .stash_push(Some("Auto-stash before switching to target branch"))
                            .map_err(|e| format!("Failed to stash changes: {}", e))?;
                    }

                    git_repo.checkout_branch(&target_branch).map_err(|e| {
                        format!("Failed to switch to branch '{}': {}", target_branch, e)
                    })?;

                    Ok::<bool, String>(needs_stash)
                })
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?
        } else {
            false
        };

        // 5. 拉取最新代码（工作区已 stash 故无需再 stash）
        spinner!("Pulling latest changes from '{}'...", target_branch)
            .with(|| safe_pull(&target_branch, &PullOptions::no_stash()))
            .map_err(|e| format!("Failed to pull latest changes: {}", e))?;

        if current_branch != target_branch {
            success!("Switched to '{}' and pulled latest", target_branch);
        } else {
            success!("Pulled latest changes from '{}'", target_branch);
        }

        // 6. 删除本地和远程源分支
        let (local_exists, remote_exists) =
            git_repo.has_branch(&source_branch).unwrap_or((false, false));

        // 6.1 删除本地分支
        // 注意：PR 已通过 GitHub API 合并（含 squash merge），Git 的 graph_descendant_of 会认为
        // squash 后的分支「未完全合并」（因 squash 创建新提交，不包含原分支提交历史）。
        // 此处直接使用 force=true，避免阻塞在 confirm 提示上。
        if local_exists {
            log_info!("Cleaning up local branch '{}'", source_branch);
            match git_repo.delete_local_branch(&source_branch, true) {
                Ok(()) => {
                    log_info!("Deleted local branch '{}'", source_branch);
                    success!("Cleaned up local branch '{}'", source_branch);
                }
                Err(e) => {
                    warning!("Failed to delete local branch '{}': {}", source_branch, e);
                }
            }
        }

        // 6.2 删除远程分支
        if remote_exists {
            spinner!("Cleaning up remote branch '{}'...", source_branch)
                .with(|| git_repo.delete_remote_branch(&source_branch))
                .map_err(|e| format!("Failed to delete remote branch: {}", e))?;
            success!("Deleted remote branch '{}'", source_branch);
        }

        // 7. 恢复 stash
        if needs_stash {
            spinner!("Restoring stashed changes...")
                .clear_on_complete(true)
                .with(|| git_repo.stash_pop(0))
                .map_err(|e| format!("Failed to restore stashed changes: {}", e))?;
            success!("Stashed changes restored");
        }

        success!("PR merge workflow completed!");

        Ok(())
    }

    /// PR 合并后更新 Jira ticket
    ///
    /// 尝试从工作历史或 PR 标题获取关联的 Jira ticket，更新状态到"已合并"，
    /// 并清理工作历史记录。
    ///
    /// # 参数
    ///
    /// * `jira_repo` - Jira 仓储
    /// * `work_history_repo` - 工作历史记录仓储
    /// * `pr_id` - PR ID
    /// * `pr_title` - PR 标题（用于提取 Jira ticket）
    /// * `repository_url` - 仓库 URL（可选）
    pub fn update_jira_after_pr_merged(
        &self,
        pr_id: &str,
        pr_title: Option<&str>,
        repository_url: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let jira_repo = get_jira_repository();
        let work_history_repo = get_jira_work_history_repository();

        // 如果没有仓库 URL，跳过工作历史相关操作
        let repo_url = repository_url.unwrap_or("");

        // 1. 尝试从工作历史读取 Jira ticket
        let mut jira_ticket = if !repo_url.is_empty() {
            work_history_repo.read_work_history(pr_id, repo_url).ok().flatten()
        } else {
            None
        };

        // 2. 如果工作历史中没有，尝试从 PR 标题提取
        if jira_ticket.is_none() {
            if let Some(title) = pr_title {
                jira_ticket = extract_jira_ticket_id(title);
            }
        }

        // 3. 如果有 Jira ticket，更新状态
        if let Some(ref ticket) = jira_ticket {
            // 读取合并时的状态配置
            if let Ok(Some(status)) = jira_repo.read_pull_request_merged_status(ticket) {
                spinner!("Updating Jira ticket {} to status: {}...", ticket, status)
                    .with(|| jira_repo.update_issue_status(ticket, &status))
                    .map_err(|e| format!("Failed to update Jira status: {}", e))?;

                success!("Jira ticket {} updated to: {}", ticket, status);
            } else {
                warning!(
                    "No Jira merged status configuration found for ticket: {}",
                    ticket
                );
            }
        } else {
            log_info!("No Jira ticket associated with this PR");
        }

        // 4. 删除工作历史记录中的 PR 条目（仅当有仓库 URL 时）
        if !repo_url.is_empty() {
            let delete_result = work_history_repo
                .delete_work_history_entry(pr_id, repo_url)
                .map_err(|e| format!("Failed to delete work history entry: {}", e))?;

            // 显示删除消息
            for message in &delete_result.messages {
                info!("{}", message);
            }

            // 显示警告信息
            for warning_msg in &delete_result.warnings {
                warning!("{}", warning_msg);
            }
        }

        Ok(())
    }
}
