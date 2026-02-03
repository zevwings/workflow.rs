//! 删除分支命令

use domain::GitError;
use prompt::{error, info, multiselect, success, warning};

use crate::registry;

/// Branch Remove 命令
pub struct BranchRemoveCommand {
    branch_name: Option<String>,
    local_only: bool,
    remote_only: bool,
    dry_run: bool,
    force: bool,
}

impl BranchRemoveCommand {
    /// 创建新的 BranchRemoveCommand
    pub fn new(
        branch_name: Option<String>,
        local_only: bool,
        remote_only: bool,
        dry_run: bool,
        force: bool,
    ) -> Self {
        Self {
            branch_name,
            local_only,
            remote_only,
            dry_run,
            force,
        }
    }

    /// 运行 `workflow branch remove` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let branch_repo = registry::get_git_repository();

        // 确定要删除的分支
        let target_branches = if let Some(name) = &self.branch_name {
            vec![name.clone()]
        } else {
            // 交互式选择分支（包含本地和远程）
            let branch_items = branch_repo
                .list_branches(false, true)
                .map_err(|e| format!("Failed to list branches: {}", e))?;

            if branch_items.is_empty() {
                error!("No branches found");
                return Err("No branches available".into());
            }

            let current_branch = branch_repo
                .get_current_branch()
                .map_err(|e| format!("Failed to get current branch: {}", e))?;

            // 获取默认分支
            let default_branch = branch_repo
                .get_default_branch()
                .map_err(|e| format!("Failed to get default branch: {}", e))?;

            // 获取忽略列表
            let config_repo = registry::get_repo_config_repository();
            let user_config = config_repo
                .load_user_config()
                .map_err(|e| format!("Failed to load user config: {}", e))
                .unwrap_or_default();
            let ignore_list = user_config.branch.ignore;

            // 过滤掉保护分支
            // 注意：name 字段现在不包含 origin/ 前缀，统一使用短名称
            let branches_to_show: Vec<_> = branch_items
                .into_iter()
                .filter(|item| {
                    let branch_name = &item.name;

                    // 过滤当前分支
                    if branch_name == &current_branch {
                        return false;
                    }
                    // 过滤默认分支
                    if branch_name == &default_branch {
                        return false;
                    }
                    // 过滤 develop/dev 分支
                    if branch_name == "develop" || branch_name == "dev" {
                        return false;
                    }
                    // 过滤 master/main 分支
                    if branch_name == "master" || branch_name == "main" {
                        return false;
                    }
                    // 过滤忽略列表中的分支
                    if ignore_list.contains(branch_name) {
                        return false;
                    }
                    true
                })
                .collect();

            if branches_to_show.is_empty() {
                info!("No branches available to remove");
                return Ok(());
            }

            // 提取显示名称用于选择
            let display_names: Vec<String> =
                branches_to_show.iter().map(|item| item.display_name.clone()).collect();

            let selected_displays = multiselect!("Select branches to remove:", display_names)
                .prompt()
                .map_err(|e| format!("Failed to select branches: {}", e))?;

            // 根据选择的显示名称找回原始分支名称
            let selected: Vec<String> = selected_displays
                .iter()
                .filter_map(|display| {
                    branches_to_show
                        .iter()
                        .find(|item| &item.display_name == display)
                        .map(|item| item.name.clone())
                })
                .collect();

            if selected.is_empty() {
                info!("No branches selected");
                return Ok(());
            }

            selected
        };

        // 获取当前分支用于检查
        let current_branch = branch_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        // 处理每个选中的分支
        let mut failed_branches: Vec<(String, String)> = Vec::new();
        // 跟踪是否对所有未合并分支使用强制删除
        let mut force_all_unmerged = self.force;
        for target_branch in target_branches {
            // 检查是否是当前分支
            if target_branch == current_branch {
                let err_msg = "Cannot remove current branch".to_string();
                error!("{} '{}'", err_msg, target_branch);
                failed_branches.push((target_branch.clone(), err_msg));
                continue;
            }

            // 检查分支是否存在
            let (exists_local, exists_remote) = branch_repo
                .has_branch(&target_branch)
                .map_err(|e| format!("Failed to check branch existence: {}", e))?;

            if !exists_local && !exists_remote {
                let err_msg = "Branch not found".to_string();
                error!("{} '{}'", err_msg, target_branch);
                failed_branches.push((target_branch.clone(), err_msg));
                continue;
            }

            // 确定要删除的范围
            let delete_local = !self.remote_only && exists_local;
            let delete_remote = !self.local_only && exists_remote;

            if !delete_local && !delete_remote {
                let err_msg = "No operation needed".to_string();
                error!("{} for branch '{}'", err_msg, target_branch);
                failed_branches.push((target_branch.clone(), err_msg));
                continue;
            }

            if self.dry_run {
                if delete_local {
                    info!("[DRY RUN] Would remove local branch '{}'", target_branch);
                }
                if delete_remote {
                    info!("[DRY RUN] Would remove remote branch '{}'", target_branch);
                }
                continue;
            }

            // 删除本地分支
            if delete_local {
                info!("Removing local branch '{}'...", target_branch);
                match branch_repo.delete_local_branch(&target_branch, self.force) {
                    Ok(()) => {
                        success!("Removed local branch '{}'", target_branch);
                    }
                    Err(e) => {
                        // 使用模式匹配精确判断错误类型
                        match e {
                            GitError::BranchNotFullyMerged(_) if !force_all_unmerged => {
                                // 提示用户是否要强制删除
                                warning!("Branch '{}' is not fully merged", target_branch);

                                // 使用 select 提供三个选项
                                let options = vec![
                                    "Force delete this branch only",
                                    "Force delete all unmerged branches",
                                    "Skip this branch",
                                ];
                                let selection = prompt::select!(
                                    "Branch '{}' is not fully merged. What would you like to do?",
                                    target_branch,
                                    options
                                )
                                .default(2)
                                .prompt()
                                .map_err(|e| format!("Failed to get selection: {}", e))?;

                                match selection.as_ref() {
                                    "Force delete this branch only" => {
                                        // 只强制删除这个分支
                                        info!("Force removing local branch '{}'...", target_branch);
                                        match branch_repo.delete_local_branch(&target_branch, true) {
                                            Ok(()) => {
                                                success!("Removed local branch '{}'", target_branch);
                                            }
                                            Err(e) => {
                                                let err_msg = format!(
                                                    "Failed to force delete local branch: {}",
                                                    e
                                                );
                                                error!(
                                                    "Failed to remove local branch '{}': {}",
                                                    target_branch, e
                                                );
                                                failed_branches.push((target_branch.clone(), err_msg));
                                                continue;
                                            }
                                        }
                                    }
                                    "Force delete all unmerged branches" => {
                                        // 设置标志，后续未合并分支都强制删除
                                        force_all_unmerged = true;
                                        info!("Force removing local branch '{}'...", target_branch);
                                        match branch_repo.delete_local_branch(&target_branch, true) {
                                            Ok(()) => {
                                                success!("Removed local branch '{}'", target_branch);
                                            }
                                            Err(e) => {
                                                let err_msg = format!(
                                                    "Failed to force delete local branch: {}",
                                                    e
                                                );
                                                error!(
                                                    "Failed to remove local branch '{}': {}",
                                                    target_branch, e
                                                );
                                                failed_branches.push((target_branch.clone(), err_msg));
                                                continue;
                                            }
                                        }
                                    }
                                    _ => {
                                        // 用户选择跳过
                                        let err_msg = "User cancelled force delete".to_string();
                                        warning!("Skipped branch '{}'", target_branch);
                                        failed_branches.push((target_branch.clone(), err_msg));
                                        continue;
                                    }
                                }
                            }
                            GitError::BranchNotFullyMerged(_) if force_all_unmerged => {
                                // 已经选择了全部强制删除，直接执行
                                warning!("Branch '{}' is not fully merged, force deleting...", target_branch);
                                match branch_repo.delete_local_branch(&target_branch, true) {
                                    Ok(()) => {
                                        success!("Removed local branch '{}'", target_branch);
                                    }
                                    Err(e) => {
                                        let err_msg =
                                            format!("Failed to force delete local branch: {}", e);
                                        error!(
                                            "Failed to remove local branch '{}': {}",
                                            target_branch, e
                                        );
                                        failed_branches.push((target_branch.clone(), err_msg));
                                        continue;
                                    }
                                }
                            }
                            _ => {
                                // 其他错误，记录并继续处理下一个分支
                                let err_msg = format!("Failed to delete local branch: {}", e);
                                error!("Failed to remove local branch '{}': {}", target_branch, e);
                                failed_branches.push((target_branch.clone(), err_msg));
                                continue;
                            }
                        }
                    }
                }
            }

            // 删除远程分支
            if delete_remote {
                info!("Removing remote branch '{}'...", target_branch);
                match branch_repo.delete_remote_branch(&target_branch) {
                    Ok(()) => {
                        success!("Removed remote branch '{}'", target_branch);
                    }
                    Err(e) => {
                        // 检查是否是因为远程分支不存在
                        if matches!(e, GitError::BranchNotFound(_)) {
                            warning!("Remote branch '{}' does not exist, skipped", target_branch);
                        } else {
                            // 其他错误才算失败
                            let err_msg = format!("Failed to delete remote branch: {}", e);
                            warning!(
                                "Failed to remove remote branch '{}': {}",
                                target_branch,
                                e
                            );
                            failed_branches.push((target_branch.clone(), err_msg));
                        }
                        // 远程分支删除失败不影响其他分支的删除，继续处理
                    }
                }
            }
        }

        if !failed_branches.is_empty() {
            // 生成详细的错误信息
            let branch_names: Vec<String> =
                failed_branches.iter().map(|(name, _)| name.clone()).collect();
            let details: Vec<String> = failed_branches
                .iter()
                .map(|(name, reason)| format!("  - {}: {}", name, reason))
                .collect();

            error!("Failed to remove branches: {}", branch_names.join(", "));
            for detail in details {
                error!("{}", detail);
            }

            Err(format!("Failed to remove branches: {}", branch_names.join(", ")).into())
        } else {
            Ok(())
        }
    }
}
