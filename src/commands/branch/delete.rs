//! Branch delete command
//!
//! Delete one or more Git branches (local and/or remote).

use crate::base::dialog::{ConfirmDialog, MultiSelectDialog};
use crate::commands::branch::helpers::sort_branches_with_priority;
use crate::commands::check;
use crate::git::GitBranch;
use crate::repo::config::RepoConfig;
use crate::{log_break, log_info, log_message, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};

/// Branch delete command
pub struct BranchDeleteCommand;

impl BranchDeleteCommand {
    /// Execute the branch delete command
    pub fn execute(
        branch_name: Option<String>,
        local_only: bool,
        remote_only: bool,
        dry_run: bool,
        force: bool,
    ) -> Result<()> {
        // 1. 运行检查
        check::CheckCommand::run_all()?;

        log_break!();
        log_message!("Branch Delete");

        // 2. 获取当前分支和默认分支
        let current_branch =
            GitBranch::current_branch().wrap_err("Failed to get current branch")?;
        log_info!("Current branch: {}", current_branch);

        let default_branch =
            GitBranch::get_default_branch().wrap_err("Failed to get default branch")?;
        log_info!("Default branch: {}", default_branch);

        // 3. 确定要删除的分支
        let branches_to_delete = if let Some(branch) = branch_name {
            vec![branch]
        } else {
            // 交互式选择
            Self::select_branches_interactively()?
        };

        if branches_to_delete.is_empty() {
            log_info!("No branches selected for deletion");
            return Ok(());
        }

        // 4. 获取分支信息并执行安全检查
        let mut branches_info = Vec::new();
        for branch_name in &branches_to_delete {
            // 检查分支是否存在
            let (exists_local, exists_remote) =
                GitBranch::is_branch_exists(branch_name).wrap_err("Failed to check branch")?;

            if !exists_local && !exists_remote {
                log_warning!("Branch '{}' does not exist", branch_name);
                continue;
            }

            // 安全检查：不能删除当前分支
            if branch_name == &current_branch {
                return Err(color_eyre::eyre::eyre!(
                    "Cannot delete current branch '{}'. Please switch to another branch first.",
                    current_branch
                ));
            }

            // 检查是否是受保护的分支
            let is_protected = Self::is_protected_branch(branch_name, &default_branch)?;

            // 检查是否已合并（仅对本地分支）
            let is_merged = if exists_local {
                GitBranch::is_branch_merged(branch_name, &default_branch)
                    .unwrap_or(false)
            } else {
                false
            };

            branches_info.push(BranchInfo {
                name: branch_name.clone(),
                exists_local,
                exists_remote,
                is_protected,
                is_merged,
            });
        }

        if branches_info.is_empty() {
            log_info!("No valid branches to delete");
            return Ok(());
        }

        // 5. 显示预览
        log_break!();
        log_message!("Branches to be deleted:");
        for info in &branches_info {
            let locations = {
                let mut locs = Vec::new();
                if info.exists_local {
                    locs.push("local");
                }
                if info.exists_remote {
                    locs.push("remote");
                }
                locs.join(" + ")
            };

            let status = if info.is_protected {
                "⚠️  PROTECTED"
            } else if !info.is_merged && info.exists_local {
                "⚠️  UNMERGED"
            } else {
                "✅"
            };

            log_info!(
                "  {} {} (locations: {})",
                status,
                info.name,
                locations
            );
        }

        // 6. Dry-run 模式
        if dry_run {
            log_break!();
            log_info!("Dry-run mode: branches will not be actually deleted");
            return Ok(());
        }

        // 7. 安全检查：受保护的分支需要额外确认
        let protected_branches: Vec<_> = branches_info
            .iter()
            .filter(|info| info.is_protected)
            .collect();

        if !protected_branches.is_empty() && !force {
            log_break!();
            log_warning!("Warning: The following branches are protected:");
            for info in &protected_branches {
                log_warning!("  - {}", info.name);
            }

            let confirmed = ConfirmDialog::new(
                "Are you sure you want to delete protected branch(es)? This is dangerous!",
            )
            .with_default(false)
            .prompt()
            .wrap_err("Failed to get user confirmation")?;

            if !confirmed {
                log_info!("Operation cancelled");
                return Ok(());
            }
        }

        // 8. 确认删除（除非使用 force）
        if !force {
            let delete_what = if local_only {
                "local"
            } else if remote_only {
                "remote"
            } else {
                "local and remote"
            };

            let unmerged_count = branches_info
                .iter()
                .filter(|info| !info.is_merged && info.exists_local)
                .count();

            let prompt = if unmerged_count > 0 {
                format!(
                    "Are you sure you want to delete {} branch(es) ({})? {} branch(es) are not merged.",
                    branches_info.len(),
                    delete_what,
                    unmerged_count
                )
            } else {
                format!(
                    "Are you sure you want to delete {} branch(es) ({})?",
                    branches_info.len(),
                    delete_what
                )
            };

            let confirmed = ConfirmDialog::new(&prompt)
                .with_default(false)
                .prompt()
                .wrap_err("Failed to get user confirmation")?;

            if !confirmed {
                log_info!("Operation cancelled");
                return Ok(());
            }
        }

        // 9. 执行删除
        let mut deleted_local = 0;
        let mut deleted_remote = 0;
        let mut failed = 0;

        for info in &branches_info {
            let branch_name = &info.name;

            // 确定删除范围
            let should_delete_local = !remote_only && info.exists_local;
            let should_delete_remote = !local_only && info.exists_remote;

            // 删除本地分支
            if should_delete_local {
                // 先尝试普通删除（如果已合并）
                let mut delete_result = if info.is_merged || force {
                    GitBranch::delete(branch_name, force)
                } else {
                    // 未合并的分支，先尝试普通删除
                    GitBranch::delete(branch_name, false)
                };

                // 如果普通删除失败且未使用 force，询问是否强制删除
                if delete_result.is_err() && !force && !info.is_merged {
                    log_break!();
                    let force_confirm = ConfirmDialog::new(format!(
                        "Branch '{}' is not merged. Force delete it?",
                        branch_name
                    ))
                    .with_default(false)
                    .prompt()
                    .wrap_err("Failed to get user confirmation")?;

                    if force_confirm {
                        delete_result = GitBranch::delete(branch_name, true);
                    }
                }

                match delete_result {
                    Ok(_) => {
                        log_success!("Deleted local branch: {}", branch_name);
                        deleted_local += 1;
                    }
                    Err(e) => {
                        log_warning!("Failed to delete local branch {}: {}", branch_name, e);
                        failed += 1;
                    }
                }
            }

            // 删除远程分支
            if should_delete_remote {
                match GitBranch::delete_remote(branch_name) {
                    Ok(_) => {
                        log_success!("Deleted remote branch: {}", branch_name);
                        deleted_remote += 1;
                    }
                    Err(e) => {
                        log_warning!("Failed to delete remote branch {}: {}", branch_name, e);
                        failed += 1;
                    }
                }
            }
        }

        // 10. 显示结果
        log_break!();
        if deleted_local > 0 || deleted_remote > 0 {
            log_success!("Deletion completed!");
            if deleted_local > 0 {
                log_info!("Deleted {} local branch(es)", deleted_local);
            }
            if deleted_remote > 0 {
                log_info!("Deleted {} remote branch(es)", deleted_remote);
            }
        }
        if failed > 0 {
            log_warning!("Failed to delete {} branch(es)", failed);
        }

        Ok(())
    }

    /// 检查是否是受保护的分支
    fn is_protected_branch(branch_name: &str, default_branch: &str) -> Result<bool> {
        // 检查是否是默认分支
        if branch_name == default_branch {
            return Ok(true);
        }

        // 检查是否是 develop 分支
        if branch_name == "develop" || branch_name == "dev" {
            return Ok(true);
        }

        // 检查是否在忽略列表中
        let ignore_branches = RepoConfig::get_ignore_branches();
        if ignore_branches.contains(&branch_name.to_string()) {
            return Ok(true);
        }

        Ok(false)
    }

    /// 交互式选择分支
    fn select_branches_interactively() -> Result<Vec<String>> {
        // 获取所有分支（本地 + 远程，去重）- 与 branch switch 保持一致
        let all_branches = GitBranch::get_all_branches(false)
            .wrap_err("Failed to get branch list")?;

        if all_branches.is_empty() {
            log_info!("No branches available");
            return Ok(Vec::new());
        }

        // 获取当前分支和默认分支（用于标记和排除）
        let current_branch = GitBranch::current_branch().unwrap_or_default();
        let default_branch = GitBranch::get_default_branch().unwrap_or_default();
        let ignore_branches = RepoConfig::get_ignore_branches();

        // 排除当前分支（不能删除当前分支）
        let mut branch_options: Vec<String> = all_branches
            .into_iter()
            .filter(|b| b != &current_branch)
            .collect();

        // 使用优先级排序 - 与 branch switch 保持一致
        branch_options = sort_branches_with_priority(branch_options)
            .wrap_err("Failed to sort branches")?;

        // 构建选项列表（添加标记信息）
        let options: Vec<String> = branch_options
            .iter()
            .map(|branch| {
                let mut label = branch.clone();

                // 标记默认分支
                if branch == &default_branch {
                    label.push_str(" (default)");
                }

                // 标记受保护的分支
                if branch == "develop" || branch == "dev" {
                    label.push_str(" (protected)");
                }
                if ignore_branches.contains(branch) {
                    label.push_str(" (protected)");
                }

                // 检查分支存在位置（本地/远程）
                let (exists_local, exists_remote) =
                    GitBranch::is_branch_exists(branch).unwrap_or((false, false));
                if exists_remote {
                    if exists_local {
                        label.push_str(" [local+remote]");
                    } else {
                        label.push_str(" [remote]");
                    }
                } else if exists_local {
                    label.push_str(" [local]");
                }

                label
            })
            .collect();

        // 多选列表
        let selected = MultiSelectDialog::new("Select branches to delete", options)
            .prompt()
            .wrap_err("Failed to select branches")?;

        if selected.is_empty() {
            return Ok(Vec::new());
        }

        // 从选中的字符串中提取分支名称（移除所有标记）
        let branch_names: Vec<String> = selected
            .iter()
            .filter_map(|s| {
                // 移除所有标记（如 " [current]", " (default)", " (protected)", " [remote]" 等）
                // 格式可能是：<branch_name> [current] (default) [remote]
                // 或者：<branch_name> (default) (protected) [local+remote]
                // 我们需要提取第一个空格之前的部分
                s.split_whitespace()
                    .next()
                    .map(|name| name.trim().to_string())
            })
            .collect();

        Ok(branch_names)
    }
}

/// 分支信息
struct BranchInfo {
    name: String,
    exists_local: bool,
    exists_remote: bool,
    is_protected: bool,
    is_merged: bool,
}
