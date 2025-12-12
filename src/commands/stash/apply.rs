//! Stash apply command
//!
//! Apply a stash entry without removing it.

use crate::base::dialog::ConfirmDialog;
use crate::commands::stash::helpers::select_stash_interactively;
use crate::git::GitStash;
use crate::{log_break, log_info, log_message, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};

/// Stash apply command
pub struct StashApplyCommand;

impl StashApplyCommand {
    /// Execute the stash apply command
    pub fn execute() -> Result<()> {
        log_break!();
        log_message!("Stash Apply");

        // 获取所有 stash 条目
        let entries = GitStash::stash_list().wrap_err("Failed to list stash entries")?;

        if entries.is_empty() {
            log_warning!("No stash entries available");
            return Ok(());
        }

        // 获取最新的 stash 信息
        let latest_stash = entries.first().unwrap();
        let latest_stash_ref = format!("stash@{{{}}}", latest_stash.index);

        // 第一步：提示是否应用最新的 stash
        let prompt = format!(
            "Apply latest stash {}?\n  Message: {}\n  Branch: {}",
            latest_stash_ref, latest_stash.message, latest_stash.branch
        );

        let use_latest = ConfirmDialog::new(&prompt)
            .with_default(true)
            .prompt()
            .wrap_err("Failed to get user confirmation")?;

        // 确定要应用的 stash
        let target_stash = if use_latest {
            latest_stash_ref
        } else {
            // 交互式选择
            select_stash_interactively()?
        };

        log_info!("Applying stash: {}", target_stash);

        // 应用 stash
        let result =
            GitStash::stash_apply(Some(&target_stash)).wrap_err("Failed to apply stash")?;

        if result.applied {
            log_success!("Stash {} applied successfully", target_stash);

            if result.has_conflicts {
                log_warning!("Merge conflicts detected!");
                log_warning!("Please resolve conflicts manually:");
                log_warning!("  1. Resolve conflicts in the affected files");
                log_warning!("  2. Stage the resolved files with: git add <file>");
                log_warning!("  3. Continue with your workflow");
            } else if let Some(stat) = result.stat {
                log_info!(
                    "Files changed: {}, insertions: {}, deletions: {}",
                    stat.files_changed,
                    stat.insertions,
                    stat.deletions
                );
            }
        } else {
            log_warning!("Failed to apply stash: {}", target_stash);
            for warning in &result.warnings {
                log_warning!("{}", warning);
            }
        }

        Ok(())
    }
}
