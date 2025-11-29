//! 分支忽略列表管理命令
//!
//! 管理分支清理时的忽略列表，支持添加、移除、列出操作。

use crate::base::util::confirm;
use crate::commands::branch::{
    add_ignore_branch, get_ignore_branches, remove_ignore_branch, save, BranchConfig,
};
use crate::git::GitRepo;
use crate::{log_break, log_info, log_message, log_success, log_warning};
use anyhow::{Context, Result};
use dialoguer::{Input, MultiSelect};

/// 分支忽略列表管理命令
pub struct BranchIgnoreCommand;

impl BranchIgnoreCommand {
    /// 添加分支到忽略列表
    pub fn add(branch_name: Option<String>) -> Result<()> {
        // 获取分支名（从参数或交互式输入）
        let branch_name = if let Some(name) = branch_name {
            name
        } else {
            Input::<String>::new()
                .with_prompt("Enter branch name to add to ignore list")
                .interact()
                .context("Failed to read branch name")?
        };

        let repo_name =
            GitRepo::extract_repo_name().context("Failed to extract repository name")?;

        let mut config = BranchConfig::load().context("Failed to load branch config")?;

        let is_new = add_ignore_branch(&mut config, repo_name.clone(), branch_name.clone())?;

        if is_new {
            save(&config).context("Failed to save branch config")?;
            log_success!(
                "Branch '{}' added to ignore list (repository: {})",
                branch_name,
                repo_name
            );
        } else {
            log_info!(
                "Branch '{}' is already in ignore list (repository: {})",
                branch_name,
                repo_name
            );
        }

        Ok(())
    }

    /// 从忽略列表移除分支
    pub fn remove(branch_name: Option<String>) -> Result<()> {
        let repo_name =
            GitRepo::extract_repo_name().context("Failed to extract repository name")?;

        let mut config = BranchConfig::load().context("Failed to load branch config")?;

        // 获取要移除的分支名（从参数或交互式选择）
        let branch_names = if let Some(name) = branch_name {
            vec![name]
        } else {
            // 获取当前仓库的忽略分支列表
            let ignore_branches = get_ignore_branches(&config, &repo_name);

            if ignore_branches.is_empty() {
                log_info!("No ignored branches found (repository: {})", repo_name);
                return Ok(());
            }

            // 构建选项列表
            let options: Vec<String> = ignore_branches.clone();

            log_break!();
            log_message!(
                "Found the following ignored branches (repository: {}):",
                repo_name
            );
            for (i, option) in options.iter().enumerate() {
                log_message!("  [{}] {}", i, option);
            }
            log_break!();

            // 使用 MultiSelect 让用户选择
            let selections = MultiSelect::new()
                .with_prompt(
                    "Select branches to remove (space to toggle, Enter to confirm, Esc to cancel)",
                )
                .items(&options)
                .interact()
                .context("Failed to get user selection")?;

            if selections.is_empty() {
                log_info!("No branches selected, operation cancelled");
                return Ok(());
            }

            log_break!();
            log_message!("Selected branches:");
            for &idx in &selections {
                log_message!("  - {}", options[idx]);
            }
            log_break!();

            // 确认删除
            let confirm_msg = format!(
                "Confirm removing {} selected branch(es) from ignore list?",
                selections.len()
            );
            if !confirm(&confirm_msg, false, Some("Operation cancelled"))? {
                return Ok(());
            }

            log_break!();

            // 收集选中的分支名
            selections.iter().map(|&idx| options[idx].clone()).collect()
        };

        // 移除选中的分支
        let mut success_count = 0;
        let mut fail_count = 0;

        for branch_name in &branch_names {
            let removed = remove_ignore_branch(&mut config, &repo_name, branch_name)?;

            if removed {
                success_count += 1;
            } else {
                log_warning!(
                    "Branch '{}' is not in ignore list (repository: {})",
                    branch_name,
                    repo_name
                );
                fail_count += 1;
            }
        }

        // 如果有成功移除的分支，保存配置
        if success_count > 0 {
            save(&config).context("Failed to save branch config")?;
            log_success!(
                "Removed {} branch(es) from ignore list (repository: {})",
                success_count,
                repo_name
            );
        }

        if fail_count > 0 {
            log_warning!("Failed to remove {} branch(es)", fail_count);
        }

        Ok(())
    }

    /// 列出当前仓库的忽略分支
    pub fn list() -> Result<()> {
        let repo_name =
            GitRepo::extract_repo_name().context("Failed to extract repository name")?;

        let config = BranchConfig::load().context("Failed to load branch config")?;

        let ignore_branches = get_ignore_branches(&config, &repo_name);

        log_break!();
        log_message!("Ignored branch list (repository: {})", repo_name);

        if ignore_branches.is_empty() {
            log_info!("No ignored branches");
        } else {
            for (index, branch) in ignore_branches.iter().enumerate() {
                log_info!("  {}. {}", index + 1, branch);
            }
            log_info!("Total: {} branch(es)", ignore_branches.len());
        }

        Ok(())
    }
}
