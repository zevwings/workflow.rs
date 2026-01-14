//! Repository cleanup command
//!
//! Clean local branches, keeping main/master, develop, current branch, and branches in ignore list.

use crate::commands::check;
use crate::git::{GitBranch, GitRepo, GitTag};
use crate::repo::config::RepoConfig;
use crate::{br, info, success, warning};
use color_eyre::{eyre::WrapErr, Result};

/// Repository cleanup command
pub struct RepoCleanCommand;

impl RepoCleanCommand {
    /// Clean local branches
    pub fn clean(dry_run: bool) -> Result<()> {
        // 1. 运行检查
        check::CheckCommand::run_all()?;

        br!();
        info!("Repository Cleanup");

        // 2. 初始化：获取当前分支、默认分支、仓库名
        let current_branch =
            GitBranch::current_branch().wrap_err("Failed to get current branch")?;
        info!("Current branch: {}", current_branch);

        let default_branch =
            GitBranch::get_default_branch().wrap_err("Failed to get default branch")?;
        info!("Default branch: {}", default_branch);

        // 获取仓库名
        let repo_name =
            GitRepo::extract_repo_name().wrap_err("Failed to extract repository name")?;
        info!("Repository: {}", repo_name);

        // 3. 清理远端引用
        info!("Cleaning remote references...");
        GitRepo::prune_remote().wrap_err("Failed to prune remote references")?;

        // 4. 读取配置文件（项目级配置）
        let ignore_branches = RepoConfig::get_ignore_branches();

        // 5. 构建排除分支列表
        let mut exclude_branches = vec![
            current_branch.clone(),
            default_branch.clone(),
            "develop".to_string(),
        ];
        exclude_branches.extend(ignore_branches);

        info!("Excluded branches: {}", exclude_branches.join(", "));

        // 6. 获取所有本地分支
        let all_branches =
            GitBranch::get_local_branches().wrap_err("Failed to get local branches")?;

        // 7. 过滤出需要删除的分支
        let branches_to_delete: Vec<String> = all_branches
            .into_iter()
            .filter(|branch| !exclude_branches.contains(branch))
            .collect();

        if branches_to_delete.is_empty() {
            success!("No branches to delete");
            return Ok(());
        }

        // 8. 分类分支（已合并 vs 未合并）
        let (merged_branches, unmerged_branches) =
            Self::classify_branches(&branches_to_delete, &default_branch)?;

        // 9. 显示预览
        br!();
        info!("Preview of branches to be deleted:");
        if !merged_branches.is_empty() {
            info!("Merged branches ({}):", merged_branches.len());
            for branch in &merged_branches {
                info!("  {}", branch);
            }
        }
        if !unmerged_branches.is_empty() {
            warning!("Unmerged branches ({}):", unmerged_branches.len());
            for branch in &unmerged_branches {
                warning!("  {}", branch);
            }
        }

        // 10. Dry-run 模式
        if dry_run {
            br!();
            info!("Dry-run mode: branches will not be actually deleted");
            return Ok(());
        }

        // 11. 确认删除
        br!();
        let total = merged_branches.len() + unmerged_branches.len();
        let prompt = format!(
            "Are you sure you want to delete {} branch(es)? (merged: {}, unmerged: {})",
            total,
            merged_branches.len(),
            unmerged_branches.len()
        );
        crate::confirm!(prompt).default(false).prompt()?;

        // 12. 删除已合并分支
        let mut deleted_count = 0;
        let mut skipped_count = 0;

        for branch in &merged_branches {
            match GitBranch::delete(branch, false) {
                Ok(()) => {
                    success!("Deleted: {}", branch);
                    deleted_count += 1;
                }
                Err(e) => {
                    warning!("Failed to delete {}: {}", branch, e);
                    skipped_count += 1;
                }
            }
        }

        // 13. 处理未合并分支
        if !unmerged_branches.is_empty() {
            br!();
            let prompt = format!(
                "There are {} unmerged branch(es), force delete them?",
                unmerged_branches.len()
            );
            if crate::confirm!(prompt).default(false).prompt()? {
                for branch in &unmerged_branches {
                    match GitBranch::delete(branch, true) {
                        Ok(()) => {
                            success!("Force deleted: {}", branch);
                            deleted_count += 1;
                        }
                        Err(e) => {
                            warning!("Failed to delete {}: {}", branch, e);
                            skipped_count += 1;
                        }
                    }
                }
            } else {
                skipped_count += unmerged_branches.len();
            }
        }

        // 14. 显示分支清理结果
        br!();
        success!("Branch cleanup completed!");
        info!("Deleted: {} branch(es)", deleted_count);
        if skipped_count > 0 {
            info!("Skipped: {} branch(es)", skipped_count);
        }

        // 15. 清理本地 tag（只存在于本地但不在远程的 tag）
        Self::clean_local_only_tags(dry_run)?;

        Ok(())
    }

    /// 清理只存在于本地但不在远程的 tag
    fn clean_local_only_tags(dry_run: bool) -> Result<()> {
        br!();
        info!("Tag Cleanup");

        // 获取所有 tag 信息
        let all_tags = GitTag::list_all_tags().wrap_err("Failed to list tags")?;

        // 筛选出只存在于本地但不在远程的 tag
        let local_only_tags: Vec<String> = all_tags
            .into_iter()
            .filter(|tag| tag.exists_local && !tag.exists_remote)
            .map(|tag| tag.name)
            .collect();

        if local_only_tags.is_empty() {
            info!("No local-only tags to clean");
            return Ok(());
        }

        // 显示预览
        br!();
        info!("Local-only tags to be deleted:");
        for tag in &local_only_tags {
            info!("  {}", tag);
        }

        // Dry-run 模式
        if dry_run {
            br!();
            info!("Dry-run mode: tags will not be actually deleted");
            return Ok(());
        }

        // 确认删除
        br!();
        let prompt = format!(
            "Are you sure you want to delete {} local-only tag(s)?",
            local_only_tags.len()
        );
        crate::confirm!(prompt).default(false).prompt()?;

        // 删除本地 tag
        let mut deleted_count = 0;
        let mut skipped_count = 0;

        for tag_name in &local_only_tags {
            match GitTag::delete_local(tag_name) {
                Ok(_) => {
                    success!("Deleted local tag: {}", tag_name);
                    deleted_count += 1;
                }
                Err(e) => {
                    warning!("Failed to delete tag {}: {}", tag_name, e);
                    skipped_count += 1;
                }
            }
        }

        // 显示结果
        br!();
        success!("Tag cleanup completed!");
        info!("Deleted: {} tag(s)", deleted_count);
        if skipped_count > 0 {
            info!("Skipped: {} tag(s)", skipped_count);
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
