//! 合并 Pull Request 命令

use crate::registry;
use color_eyre::Result;
use prompt::{info, spinner, success, warning};

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

        if self.force {
            info!("Force mode enabled: remote branch will be deleted after merge");
        }

        // 2. 合并 PR（force 参数控制是否删除远程分支）
        spinner!("Merging PR #{}...", self.pr_id)
            .with(|| pr_service.merge_pull_request(&self.pr_id, self.force))
            .map_err(|e| format!("Failed to merge Pull Request: {}", e))?;

        success!("Pull Request #{} merged successfully!", self.pr_id);

        // 3. 切换到 target_branch
        if current_branch != target_branch {
            info!("Switching to branch '{}'...", target_branch);
            git_repo
                .checkout_branch(&target_branch)
                .map_err(|e| format!("Failed to switch to branch '{}': {}", target_branch, e))?;
            success!("Switched to branch '{}'", target_branch);
        }

        // 4. 拉取最新代码
        spinner!("Pulling latest changes from '{}'...", target_branch)
            .with(|| git_repo.pull(&target_branch))
            .map_err(|e| format!("Failed to pull latest changes: {}", e))?;
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

        success!("PR merge workflow completed!");

        Ok(())
    }
}
