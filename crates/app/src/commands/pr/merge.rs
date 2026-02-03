//! 合并 Pull Request 命令

use crate::registry;
use color_eyre::Result;
<<<<<<< HEAD
use prompt::{error, info, spinner, success, warning, Spinner};
=======
use prompt::{info, spinner, success, warning};
>>>>>>> f7b652fb4c2f362a3748a90402b3e98060d4f4f6

/// Pull Request Merge 命令
pub struct PullRequestMergeCommand {
    pr_id: String,
    force: bool,
}

impl PullRequestMergeCommand {
    /// 创建新的 PullRequestMergeCommand
    pub fn new(pr_id: String, force: bool) -> Self {
        Self { pr_id, force }
    }

    /// 运行 `workflow pr merge` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pr_service = registry::get_pull_request_service();
        let git_repo = registry::get_git_repository();
<<<<<<< HEAD
=======

        // 1. 获取 PR 信息（包含 source_branch, target_branch）
        let pr_info = spinner!("Fetching PR #{} info...", self.pr_id)
            .with(|| pr_service.get_pull_request(&self.pr_id))
            .map_err(|e| format!("Failed to get Pull Request info: {}", e))?;

        let source_branch = pr_info.source_branch.clone();
        let target_branch = pr_info.target_branch.clone();
        let current_branch = git_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        info!(
            "Merging '{}' into '{}'...",
            source_branch, target_branch
        );
>>>>>>> f7b652fb4c2f362a3748a90402b3e98060d4f4f6

        if self.force {
            info!("Force mode enabled: remote branch will be deleted after merge");
        }

<<<<<<< HEAD
        // 1. 获取 PR 信息（在合并前获取源分支和目标分支）
        let pr_info = pr_service
            .get_pull_request(&self.pr_id)
            .map_err(|e| format!("Failed to get PR info: {}", e))?;
        let source_branch = pr_info.source_branch;
        let target_branch = pr_info.target_branch;

        // 2. 合并 PR
        Spinner::new(format!("Merging PR #{}...", self.pr_id))
=======
        // 2. 合并 PR（force 参数控制是否删除远程分支）
        spinner!("Merging PR #{}...", self.pr_id)
>>>>>>> f7b652fb4c2f362a3748a90402b3e98060d4f4f6
            .with(|| pr_service.merge_pull_request(&self.pr_id, self.force))
            .map_err(|e| format!("Failed to merge Pull Request: {}", e))?;

        success!("Pull Request #{} merged successfully!", self.pr_id);

        // 3. 切换到 target_branch
<<<<<<< HEAD
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

=======
        if current_branch != target_branch {
>>>>>>> f7b652fb4c2f362a3748a90402b3e98060d4f4f6
            info!("Switching to branch '{}'...", target_branch);
            git_repo
                .checkout_branch(&target_branch)
                .map_err(|e| format!("Failed to switch to branch '{}': {}", target_branch, e))?;
            success!("Switched to branch '{}'", target_branch);
<<<<<<< HEAD

            needs_stash
        } else {
            false
        };

        // 4. 拉取最新代码
        info!("Pulling latest changes from '{}'...", target_branch);
        if let Err(e) = git_repo.pull(&target_branch) {
            let error_msg = e.to_string();
            // 检测是否是冲突错误
            if error_msg.contains("conflict")
                || error_msg.contains("Conflict")
                || error_msg.contains("CONFLICT")
            {
                error!("Pull failed due to merge conflicts!");
                error!("Please resolve the conflicts manually:");
                info!("  1. Edit the conflicting files to resolve conflicts");
                info!("  2. Run 'git add <resolved-files>'");
                info!("  3. Run 'git commit' to complete the merge");
                info!("  Or run 'git merge --abort' to cancel the merge");
                return Err(
                    format!("Pull failed: merge conflicts detected - {}", error_msg).into(),
                );
            }
            return Err(format!("Failed to pull latest changes: {}", error_msg).into());
        }
=======
        }

        // 4. 拉取最新代码
        spinner!("Pulling latest changes from '{}'...", target_branch)
            .with(|| git_repo.pull(&target_branch))
            .map_err(|e| format!("Failed to pull latest changes: {}", e))?;
>>>>>>> f7b652fb4c2f362a3748a90402b3e98060d4f4f6
        success!("Pulled latest changes from '{}'", target_branch);

        // 5. 删除本地源分支（如果存在且不是当前分支）
        let (local_exists, _) = git_repo
            .has_branch(&source_branch)
            .unwrap_or((false, false));

        if local_exists {
            info!("Cleaning up local branch '{}'...", source_branch);
            match git_repo.delete_branch(&source_branch, false) {
                Ok(()) => {
                    success!("Deleted local branch '{}'", source_branch);
                }
                Err(e) => {
                    warning!(
                        "Failed to delete local branch '{}': {}",
                        source_branch, e
                    );
                }
            }
        }

<<<<<<< HEAD
        // 6. 恢复 stash
        if needs_stash {
            info!("Restoring stashed changes...");
            git_repo
                .stash_pop(0)
                .map_err(|e| format!("Failed to restore stashed changes: {}", e))?;
            success!("Stashed changes restored");
        }

=======
>>>>>>> f7b652fb4c2f362a3748a90402b3e98060d4f4f6
        success!("PR merge workflow completed!");

        Ok(())
    }
}
