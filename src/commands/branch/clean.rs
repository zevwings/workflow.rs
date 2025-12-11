//! 分支清理命令
//!
//! 清理本地分支，保留 main/master、develop 和当前分支，以及配置文件中的忽略分支。

use crate::base::dialog::ConfirmDialog;
use crate::branch::config::BranchConfig;
use crate::commands::check;
use crate::git::{GitBranch, GitRepo};
use crate::{log_break, log_info, log_message, log_success, log_warning};
use anyhow::{Context, Result};

/// 分支清理命令
pub struct BranchCleanCommand;

impl BranchCleanCommand {
    /// 清理本地分支
    pub fn clean(dry_run: bool) -> Result<()> {
        // 1. 运行检查
        check::CheckCommand::run_all()?;

        log_break!();
        log_message!("Branch Cleanup");

        // 2. 初始化：获取当前分支、默认分支、仓库名
        let current_branch = GitBranch::current_branch().context("Failed to get current branch")?;
        log_info!("Current branch: {}", current_branch);

        let default_branch =
            GitBranch::get_default_branch().context("Failed to get default branch")?;
        log_info!("Default branch: {}", default_branch);

        // 获取仓库名
        let repo_name =
            GitRepo::extract_repo_name().context("Failed to extract repository name")?;
        log_info!("Repository: {}", repo_name);

        // 3. 清理远端引用
        log_info!("Cleaning remote references...");
        GitRepo::prune_remote().context("Failed to prune remote references")?;

        // 4. 读取配置文件
        let config = BranchConfig::load().context("Failed to load branch config")?;
        let ignore_branches = config.get_ignore_branches(&repo_name);

        // 5. 构建排除分支列表
        let mut exclude_branches = vec![
            current_branch.clone(),
            default_branch.clone(),
            "develop".to_string(),
        ];
        exclude_branches.extend(ignore_branches);

        log_info!("Excluded branches: {}", exclude_branches.join(", "));

        // 6. 获取所有本地分支
        let all_branches =
            GitBranch::get_local_branches().context("Failed to get local branches")?;

        // 7. 过滤出需要删除的分支
        let branches_to_delete: Vec<String> = all_branches
            .into_iter()
            .filter(|branch| !exclude_branches.contains(branch))
            .collect();

        if branches_to_delete.is_empty() {
            log_success!("No branches to delete");
            return Ok(());
        }

        // 8. 分类分支（已合并 vs 未合并）
        let (merged_branches, unmerged_branches) =
            Self::classify_branches(&branches_to_delete, &default_branch)?;

        // 9. 显示预览
        log_break!();
        log_message!("Preview of branches to be deleted:");
        if !merged_branches.is_empty() {
            log_info!("Merged branches ({}):", merged_branches.len());
            for branch in &merged_branches {
                log_info!("  {}", branch);
            }
        }
        if !unmerged_branches.is_empty() {
            log_warning!("Unmerged branches ({}):", unmerged_branches.len());
            for branch in &unmerged_branches {
                log_warning!("  {}", branch);
            }
        }

        // 10. Dry-run 模式
        if dry_run {
            log_break!();
            log_info!("Dry-run mode: branches will not be actually deleted");
            return Ok(());
        }

        // 11. 确认删除
        log_break!();
        let total = merged_branches.len() + unmerged_branches.len();
        let prompt = format!(
            "Are you sure you want to delete {} branch(es)? (merged: {}, unmerged: {})",
            total,
            merged_branches.len(),
            unmerged_branches.len()
        );
        ConfirmDialog::new(&prompt)
            .with_default(false)
            .with_cancel_message("Operation cancelled")
            .prompt()?;

        // 12. 删除已合并分支
        let mut deleted_count = 0;
        let mut skipped_count = 0;

        for branch in &merged_branches {
            match GitBranch::delete(branch, false) {
                Ok(()) => {
                    log_success!("Deleted: {}", branch);
                    deleted_count += 1;
                }
                Err(e) => {
                    log_warning!("Failed to delete {}: {}", branch, e);
                    skipped_count += 1;
                }
            }
        }

        // 13. 处理未合并分支
        if !unmerged_branches.is_empty() {
            log_break!();
            let prompt = format!(
                "There are {} unmerged branch(es), force delete them?",
                unmerged_branches.len()
            );
            if ConfirmDialog::new(&prompt).with_default(false).prompt()? {
                for branch in &unmerged_branches {
                    match GitBranch::delete(branch, true) {
                        Ok(()) => {
                            log_success!("Force deleted: {}", branch);
                            deleted_count += 1;
                        }
                        Err(e) => {
                            log_warning!("Failed to delete {}: {}", branch, e);
                            skipped_count += 1;
                        }
                    }
                }
            } else {
                skipped_count += unmerged_branches.len();
            }
        }

        // 14. 显示结果
        log_break!();
        log_success!("Cleanup completed!");
        log_info!("Deleted: {} branch(es)", deleted_count);
        if skipped_count > 0 {
            log_info!("Skipped: {} branch(es)", skipped_count);
        }

        Ok(())
    }

    /// 分类分支（已合并 vs 未合并）
    fn classify_branches(
        branches: &[String],
        base_branch: &str,
    ) -> Result<(Vec<String>, Vec<String>)> {
        let mut merged = Vec::new();
        let mut unmerged = Vec::new();

        for branch in branches {
            if GitBranch::is_branch_merged(branch, base_branch)? {
                merged.push(branch.clone());
            } else {
                unmerged.push(branch.clone());
            }
        }

        Ok((merged, unmerged))
    }
}
