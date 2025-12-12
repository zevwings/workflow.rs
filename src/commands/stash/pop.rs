//! Stash pop command
//!
//! Apply a stash entry and remove it.

use crate::base::dialog::ConfirmDialog;
use crate::commands::stash::helpers::select_stash_interactively;
use crate::git::GitStash;
use crate::{log_break, log_info, log_message, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};

/// Stash pop command
pub struct StashPopCommand;

impl StashPopCommand {
    /// Execute the stash pop command
    pub fn execute() -> Result<()> {
        log_break!();
        log_message!("Stash Pop");

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
            "Pop latest stash {}?\n  Message: {}\n  Branch: {}",
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

        log_info!("Popping stash: {}", target_stash);

        // 应用并删除 stash
        let result = GitStash::stash_pop(Some(&target_stash)).wrap_err("Failed to pop stash")?;

        if result.restored {
            log_success!("Stash {} applied and removed", target_stash);
            if let Some(msg) = result.message {
                log_info!("{}", msg);
            }
        } else {
            log_warning!("Failed to apply stash: {}", target_stash);
            log_warning!("The stash entry is kept due to conflicts or errors.");
            for warning in &result.warnings {
                log_warning!("{}", warning);
            }
        }

        Ok(())
    }
}
