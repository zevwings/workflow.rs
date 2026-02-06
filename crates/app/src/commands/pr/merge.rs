//! 合并 Pull Request 命令

use domain::GitError;
use prompt::{confirm, error, info, input, spinner, success, validators, warning};

use crate::registry;
use crate::workflows::utils::update_jira_after_pr_merged;

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
        let pr_service = registry::get_pull_request_service();
        let git_repo = registry::get_git_repository();

        if self.force {
            info!("Force mode enabled: remote branch will be deleted after merge");
        }

        let pr_id = if let Some(pr_id) = &self.pr_id {
            pr_id.clone()
        } else {
            // 带验证的输入（使用 regex 验证邮箱）
            let pr_id_validator =
                validators::regex(r"^[0-9]+$", Some("Please enter a valid PR ID"))
                    .map_err(|e| format!("Invalid PR ID regex: {}", e))?;
            // 交互式输入
            let input_id = input!("Please enter your PR ID:")
                .validator(pr_id_validator)
                .prompt()
                .map_err(|e| format!("Failed to get PR ID: {}", e))?;

            if input_id.trim().is_empty() {
                return Err("PR ID is required".into());
            }
            input_id.trim().to_string()
        };

        // 1. 获取 PR 信息（在合并前获取源分支、目标分支和标题）
        let pr_info = pr_service
            .get_pull_request(&pr_id)
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
        update_jira_after_pr_merged(&pr_id, Some(pr_title.as_str()), repository_url)?;

        // 4. 切换到 target_branch
        let current_branch = git_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        let needs_stash = if current_branch != target_branch {
            // 检查是否有未提交的更改
            let status = git_repo
                .get_working_tree_status()
                .map_err(|e| format!("Failed to get status: {}", e))?;

            let needs_stash = !status.is_clean();

            if needs_stash {
                info!("Stashing uncommitted changes before switching branch...");
                git_repo
                    .stash_push(Some("Auto-stash before switching to target branch"))
                    .map_err(|e| format!("Failed to stash changes: {}", e))?;
            }

            info!("Switching to branch '{}'...", target_branch);
            git_repo
                .checkout_branch(&target_branch)
                .map_err(|e| format!("Failed to switch to branch '{}': {}", target_branch, e))?;
            success!("Switched to branch '{}'", target_branch);

            needs_stash
        } else {
            false
        };

        // 5. 拉取最新代码
        info!("Pulling latest changes from '{}'...", target_branch);
        if let Err(e) = git_repo.pull(&target_branch) {
            if matches!(e, GitError::MergeConflict) {
                error!("Pull failed due to merge conflicts!");
                error!("Please resolve the conflicts manually:");
                info!("  1. Edit the conflicting files to resolve conflicts");
                info!("  2. Run 'git add <resolved-files>'");
                info!("  3. Run 'git commit' to complete the merge");
                info!("  Or run 'git merge --abort' to cancel the merge");
                return Err(format!("Pull failed: merge conflicts detected - {}", e).into());
            }
            return Err(format!("Failed to pull latest changes: {}", e).into());
        }
        success!("Pulled latest changes from '{}'", target_branch);

        // 6. 删除本地和远程源分支
        let (local_exists, remote_exists) =
            git_repo.has_branch(&source_branch).unwrap_or((false, false));

        // 6.1 删除本地分支
        if local_exists {
            info!("Cleaning up local branch '{}'...", source_branch);
            match git_repo.delete_local_branch(&source_branch, false) {
                Ok(()) => {
                    success!("Deleted local branch '{}'", source_branch);
                }
                Err(e) => {
                    // 使用模式匹配精确判断错误类型
                    match e {
                        GitError::BranchNotFullyMerged(_) => {
                            warning!("Branch '{}' is not fully merged", source_branch);
                            let force_delete = confirm!("Force delete branch '{}'?", source_branch)
                                .default(false)
                                .prompt()
                                .unwrap_or(false);

                            if force_delete {
                                match git_repo.delete_local_branch(&source_branch, true) {
                                    Ok(()) => {
                                        success!("Force deleted local branch '{}'", source_branch);
                                    }
                                    Err(e) => {
                                        warning!(
                                            "Failed to force delete local branch '{}': {}",
                                            source_branch,
                                            e
                                        );
                                    }
                                }
                            } else {
                                info!("Skipped deleting branch '{}'", source_branch);
                            }
                        }
                        _ => {
                            warning!("Failed to delete local branch '{}': {}", source_branch, e);
                        }
                    }
                }
            }
        }

        // 6.2 删除远程分支
        if remote_exists {
            info!("Cleaning up remote branch '{}'...", source_branch);
            match git_repo.delete_remote_branch(&source_branch) {
                Ok(()) => {
                    success!("Deleted remote branch '{}'", source_branch);
                }
                Err(GitError::BranchNotFound(_)) => {
                    // 远程分支可能已被 GitHub 自动删除
                    info!(
                        "Remote branch '{}' already deleted (may have been auto-deleted by GitHub)",
                        source_branch
                    );
                }
                Err(e) => {
                    warning!("Failed to delete remote branch '{}': {}", source_branch, e);
                }
            }
        }

        // 7. 恢复 stash
        if needs_stash {
            info!("Restoring stashed changes...");
            git_repo
                .stash_pop(0)
                .map_err(|e| format!("Failed to restore stashed changes: {}", e))?;
            success!("Stashed changes restored");
        }

        success!("PR merge workflow completed!");

        Ok(())
    }
}
