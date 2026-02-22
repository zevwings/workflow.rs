//! 重命名分支命令

use domain::GitError;
use prompt::{confirm, error, info, input, select, success, warning};

use crate::bootstrap;
use crate::util::safe_push;

/// Branch Rename 命令
pub struct BranchRenameCommand;

impl Default for BranchRenameCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl BranchRenameCommand {
    /// 创建新的 BranchRenameCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow branch rename` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let branch_repo = bootstrap::get_git_repository();

        // 列出所有本地分支
        let branch_items = branch_repo
            .list_branches(false, false)
            .map_err(|e| format!("Failed to list branches: {}", e))?;

        if branch_items.is_empty() {
            error!("No branches found");
            return Err("No branches available".into());
        }

        // 提取分支名称用于选择（本地分支的 name 和 display_name 相同）
        let branch_names: Vec<String> = branch_items.into_iter().map(|item| item.name).collect();

        // 交互式选择要重命名的分支
        let old_branch = select!("Select branch to rename:", branch_names)
            .prompt()
            .map_err(|e| format!("Failed to select branch: {}", e))?;

        // 输入新分支名
        let new_branch = input!("Please enter your new branch name:")
            .default(&old_branch)
            .prompt()
            .map_err(|e| format!("Failed to get new branch name: {}", e))?;

        if new_branch == old_branch {
            info!("Branch name unchanged");
            return Ok(());
        }

        // 检查新分支名是否已存在
        let (exists_local, exists_remote) = branch_repo
            .has_branch(&new_branch)
            .map_err(|e| format!("Failed to check branch existence: {}", e))?;

        if exists_local || exists_remote {
            error!("Branch '{}' already exists", new_branch);
            return Err(format!("Branch '{}' already exists", new_branch).into());
        }

        // 检查旧分支是否有远程分支
        let (_, has_remote) = branch_repo
            .has_branch(&old_branch)
            .map_err(|e| format!("Failed to check branch existence: {}", e))?;

        // 重命名本地分支
        info!("Renaming branch '{}' to '{}'...", old_branch, new_branch);
        branch_repo
            .rename_branch(Some(&old_branch), &new_branch)
            .map_err(|e| format!("Failed to rename branch: {}", e))?;

        // 如果有远程分支，处理远程分支
        if has_remote {
            let should_update_remote = confirm!(
                "Remote branch 'origin/{}' exists. Update remote?",
                old_branch
            )
            .default(true)
            .prompt()
            .map_err(|e| format!("Failed to get confirmation: {}", e))?;

            if should_update_remote {
                if let Err(e) = safe_push(&new_branch, true) {
                    error!("Failed to push new branch: {}", e);
                    warning!("Local branch renamed, but remote update failed");
                    return Err(format!("Failed to push new branch: {}", e).into());
                }

                // 删除远程旧分支
                if let Err(e) = branch_repo.delete_remote_branch(&old_branch) {
                    // 忽略远程分支不存在的错误
                    if !matches!(e, GitError::BranchNotFound(_)) {
                        warning!("Failed to delete remote branch: {}", e);
                    }
                }

                success!(
                    "Renamed '{}' to '{}' (local and remote)",
                    old_branch,
                    new_branch
                );
            } else {
                success!("Renamed '{}' to '{}' (local only)", old_branch, new_branch);
            }
        } else {
            success!("Renamed '{}' to '{}'", old_branch, new_branch);
        }

        Ok(())
    }
}
