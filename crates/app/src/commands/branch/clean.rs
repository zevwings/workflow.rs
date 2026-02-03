//! 清理本地分支命令

use color_eyre::Result;
use domain::GitError;
use prompt::{confirm, error, info, success, warning};

use crate::registry;

/// Branch Clean 命令
pub struct BranchCleanCommand {
    dry_run: bool,
}

impl BranchCleanCommand {
    /// 创建新的 BranchCleanCommand
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    /// 运行 `workflow branch clean` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let branch_repo = registry::get_git_repository();

        // 获取默认分支
        let default_branch = branch_repo
            .get_default_branch()
            .map_err(|e| format!("Failed to get default branch: {}", e))?;

        // 获取当前分支
        let current_branch = branch_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        // 获取忽略列表
        let config_repo = registry::get_repo_config_repository();

        let user_config = config_repo
            .load_user_config()
            .map_err(|e| format!("Failed to load user config: {}", e))
            .unwrap_or_default();
        let ignore_list = user_config.branch.ignore;

        // 获取所有本地分支
        let branch_items = branch_repo
            .list_branches(false, false)
            .map_err(|e| format!("Failed to list branches: {}", e))?;

        // 过滤需要保留的分支
        let branches_to_clean: Vec<String> = branch_items
            .into_iter()
            .filter(|item| {
                let branch_name = &item.name;
                // 保留默认分支
                if branch_name == &default_branch {
                    return false;
                }
                // 保留当前分支
                if branch_name == &current_branch {
                    return false;
                }
                // 保留 develop 分支（如果存在）
                if branch_name == "develop" || branch_name == "dev" {
                    return false;
                }
                // 保留忽略列表中的分支
                if ignore_list.contains(branch_name) {
                    return false;
                }
                true
            })
            .map(|item| item.name)
            .collect();

        if branches_to_clean.is_empty() {
            info!("No branches to clean");
            return Ok(());
        }

        // 显示将要删除的分支
        info!("Found {} branch(es) to clean:", branches_to_clean.len());
        for branch in &branches_to_clean {
            info!("  - {}", branch);
        }

        if self.dry_run {
            info!(
                "[DRY RUN] Would delete {} branch(es)",
                branches_to_clean.len()
            );
            return Ok(());
        }

        // 确认删除
        let confirmed = confirm!("Delete {} branch(es)?", branches_to_clean.len())
            .default(true)
            .prompt()
            .map_err(|e| format!("Failed to get confirmation: {}", e))?;

        if !confirmed {
            info!("Operation cancelled");
            return Ok(());
        }

        // 删除所有可清理的分支
        let mut deleted_count = 0;
        let mut failed_branches = Vec::new();

        for branch in &branches_to_clean {
            match branch_repo.delete_local_branch(branch, false) {
                Ok(()) => {
                    deleted_count += 1;
                    info!("Deleted branch '{}'", branch);
                }
                Err(e) => {
                    // 使用模式匹配精确判断错误类型
                    match e {
                        GitError::BranchNotFullyMerged(_) => {
                            warning!("Branch '{}' is not fully merged", branch);
                            let force_delete = confirm!("Force delete branch '{}'?", branch)
                                .default(false)
                                .prompt()
                                .map_err(|e| format!("Failed to get confirmation: {}", e))?;

                            if force_delete {
                                match branch_repo.delete_local_branch(branch, true) {
                                    Ok(()) => {
                                        deleted_count += 1;
                                        info!("Force deleted branch '{}'", branch);
                                    }
                                    Err(e) => {
                                        warning!(
                                            "Failed to force delete branch '{}': {}",
                                            branch,
                                            e
                                        );
                                        failed_branches.push((branch.clone(), e));
                                    }
                                }
                            } else {
                                info!("Skipped branch '{}'", branch);
                            }
                        }
                        _ => {
                            warning!("Failed to delete branch '{}': {}", branch, e);
                            failed_branches.push((branch.clone(), e));
                        }
                    }
                }
            }
        }

        if deleted_count > 0 {
            success!("Cleaned {} branch(es)", deleted_count);
        }

        if !failed_branches.is_empty() {
            warning!("Failed to delete {} branch(es)", failed_branches.len());
            for (branch, error) in failed_branches {
                error!("  - {}: {}", branch, error);
            }
        }

        Ok(())
    }
}
