//! Stash apply command
//!
//! Apply a stash entry without removing it.

use crate::commands::stash::helpers::select_stash_interactively;
use crate::services::git::GitStash;
use crate::{br, info, success, warning};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};

/// Stash apply command
pub struct StashApplyCommand;

impl StashApplyCommand {
    /// Execute the stash apply command
    pub fn execute() -> Result<()> {
        br!();
        info!("Stash Apply");

        // 获取所有 stash 条目
        let entries = GitStash::stash_list().wrap_err("Failed to list stash entries")?;

        if entries.is_empty() {
            warning!("No stash entries available");
            return Ok(());
        }

        // 获取最新的 stash 信息
        let latest_stash = entries.first().ok_or_else(|| eyre!("No stash entries available"))?;
        let latest_stash_ref = format!("stash@{{{}}}", latest_stash.index);

        // 第一步：提示是否应用最新的 stash
        let prompt = format!(
            "Apply latest stash {}?\n  Message: {}\n  Branch: {}",
            latest_stash_ref, latest_stash.message, latest_stash.branch
        );

        let use_latest = crate::confirm!(prompt)
            .default(true)
            .prompt()
            .wrap_err("Failed to get user confirmation")?;

        // 确定要应用的 stash
        let target_stash = if use_latest {
            latest_stash_ref
        } else {
            // 交互式选择
            select_stash_interactively()?
        };

        info!("Applying stash: {}", target_stash);

        // 应用 stash
        let result =
            GitStash::stash_apply(Some(&target_stash)).wrap_err("Failed to apply stash")?;

        if result.applied {
            success!("Stash {} applied successfully", target_stash);

            if result.has_conflicts {
                warning!("Merge conflicts detected!");
                warning!("Please resolve conflicts manually:");
                warning!("  1. Resolve conflicts in the affected files");
                warning!("  2. Stage the resolved files with: git add <file>");
                warning!("  3. Continue with your workflow");
            } else if let Some(stat) = result.stat {
                info!(
                    "Files changed: {}, insertions: {}, deletions: {}",
                    stat.files_changed, stat.insertions, stat.deletions
                );
            }
        } else {
            warning!("Failed to apply stash: {}", target_stash);
            for warning in &result.warnings {
                warning!("{}", warning);
            }
        }

        Ok(())
    }
}
