//! 分支忽略列表管理命令
//!
//! 管理分支清理时的忽略列表，支持添加、移除、列出操作。
//! 配置保存在个人偏好配置（~/.workflow/config/repository.toml）中，不提交到 Git。

use crate::base::dialog::{ConfirmDialog, InputDialog, MultiSelectDialog};
use crate::base::table::{TableBuilder, TableStyle};
use crate::git::BranchRow;
use crate::repo::config::RepoConfig;
use crate::{log_break, log_info, log_message, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};

/// 分支忽略列表管理命令
pub struct BranchIgnoreCommand;

impl BranchIgnoreCommand {
    /// 添加分支到忽略列表
    ///
    /// 保存到个人偏好配置（~/.workflow/config/repository.toml）
    pub fn add(branch_name: Option<String>) -> Result<()> {
        // 获取分支名（从参数或交互式输入）
        let branch_name = if let Some(name) = branch_name {
            name
        } else {
            InputDialog::new("Enter branch name to add to ignore list")
                .prompt()
                .wrap_err("Failed to read branch name")?
        };

        // 加载统一配置
        let mut config = RepoConfig::load().wrap_err("Failed to load repository config")?;

        // 获取或创建分支配置
        let branch_config = config.branch.get_or_insert_with(Default::default);

        // 检查是否已存在
        if branch_config.ignore.contains(&branch_name) {
            log_info!("Branch '{}' is already in ignore list", branch_name);
            return Ok(());
        }

        // 添加到忽略列表
        branch_config.ignore.push(branch_name.clone());

        // 保存配置
        config.save().wrap_err("Failed to save repository config")?;

        log_success!(
            "Branch '{}' added to ignore list (personal preference)",
            branch_name
        );
        log_info!("Configuration saved to ~/.workflow/config/repository.toml");

        Ok(())
    }

    /// 从忽略列表移除分支
    ///
    /// 从个人偏好配置（~/.workflow/config/repository.toml）中移除
    pub fn remove(branch_name: Option<String>) -> Result<()> {
        // 加载统一配置
        let mut config = RepoConfig::load().wrap_err("Failed to load repository config")?;

        // 获取当前仓库的忽略分支列表
        let ignore_branches = RepoConfig::get_ignore_branches();

        // 获取要移除的分支名（从参数或交互式选择）
        let branch_names = if let Some(name) = branch_name {
            vec![name]
        } else {
            if ignore_branches.is_empty() {
                log_info!("No ignored branches found");
                return Ok(());
            }

            // 构建选项列表
            let options: Vec<String> = ignore_branches.clone();

            log_break!();
            log_message!("Found the following ignored branches:");
            for (i, option) in options.iter().enumerate() {
                log_message!("  [{}] {}", i, option);
            }
            log_break!();

            // 使用 MultiSelect 让用户选择
            let options_vec: Vec<String> = options.to_vec();
            let selected_branches = MultiSelectDialog::new(
                "Select branches to remove (space to toggle, Enter to confirm, Esc to cancel)",
                options_vec,
            )
            .prompt()
            .wrap_err("Failed to get user selection")?;

            if selected_branches.is_empty() {
                log_info!("No branches selected, operation cancelled");
                return Ok(());
            }

            log_break!();
            log_message!("Selected branches:");
            for branch in &selected_branches {
                log_message!("  - {}", branch);
            }

            let selections: Vec<usize> = options
                .iter()
                .enumerate()
                .filter_map(|(i, opt)| {
                    if selected_branches.contains(opt) {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();
            log_break!();

            // 确认删除
            let confirm_msg = format!(
                "Confirm removing {} selected branch(es) from ignore list?",
                selections.len()
            );
            if !ConfirmDialog::new(&confirm_msg)
                .with_default(false)
                .with_cancel_message("Operation cancelled")
                .prompt()?
            {
                return Ok(());
            }

            log_break!();

            // 收集选中的分支名
            selections.iter().map(|&idx| options[idx].clone()).collect()
        };

        // 获取或创建分支配置
        let branch_config = config.branch.get_or_insert_with(Default::default);

        // 移除选中的分支
        let mut success_count = 0;
        let mut fail_count = 0;

        for branch_name in &branch_names {
            if let Some(pos) = branch_config.ignore.iter().position(|b| b == branch_name) {
                branch_config.ignore.remove(pos);
                success_count += 1;
            } else {
                log_warning!("Branch '{}' is not in ignore list", branch_name);
                fail_count += 1;
            }
        }

        // 如果有成功移除的分支，保存配置
        if success_count > 0 {
            RepoConfig::save(&config).wrap_err("Failed to save repository config")?;
            log_success!(
                "Removed {} branch(es) from ignore list (personal preference)",
                success_count
            );
            log_info!("Configuration saved to ~/.workflow/config/repository.toml");
        }

        if fail_count > 0 {
            log_warning!("Failed to remove {} branch(es)", fail_count);
        }

        Ok(())
    }

    /// 列出当前仓库的忽略分支
    ///
    /// 从个人偏好配置中读取
    pub fn list() -> Result<()> {
        // 从项目级配置读取忽略分支列表
        let ignore_branches = RepoConfig::get_ignore_branches();

        if ignore_branches.is_empty() {
            log_info!("No ignored branches found");
            return Ok(());
        }

        // 构建表格数据
        let rows: Vec<BranchRow> = ignore_branches
            .iter()
            .enumerate()
            .map(|(index, branch)| BranchRow {
                index: (index + 1).to_string(),
                name: branch.clone(),
            })
            .collect();

        // 使用表格显示
        log_message!(
            "{}",
            TableBuilder::new(rows)
                .with_title("Ignored Branches")
                .with_style(TableStyle::Modern)
                .render()
        );

        log_info!("\nTotal: {} branch(es)", ignore_branches.len());

        Ok(())
    }
}
