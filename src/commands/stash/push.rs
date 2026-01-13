//! Stash push command
//!
//! Save current working directory changes to stash.

use crate::git::{GitCommit, GitStash};
use crate::{br, info, success, warning};
use color_eyre::{eyre::WrapErr, Result};

/// Stash push command
pub struct StashPushCommand;

impl StashPushCommand {
    /// Execute the stash push command
    ///
    /// Saves current working directory and staged changes to stash.
    /// Prompts for an optional message to identify the stash entry.
    pub fn execute() -> Result<()> {
        br!();
        info!("Stash Push");

        // 检查是否有未提交的更改
        let has_changes =
            GitCommit::has_commit().wrap_err("Failed to check working directory status")?;

        if !has_changes {
            warning!("No changes to stash. Working tree is clean.");
            return Ok(());
        }

        // 提示用户输入 stash 消息（可选）
        let message = crate::input!("Stash message (optional, press Enter to skip)")
            .prompt()
            .wrap_err("Failed to get stash message")?;

        // 执行 stash push
        let stash_message = if message.trim().is_empty() {
            None
        } else {
            Some(message.trim())
        };

        info!("Stashing changes...");
        GitStash::stash_push(stash_message).wrap_err("Failed to stash changes")?;

        if let Some(msg) = stash_message {
            success!("Changes stashed with message: {}", msg);
        } else {
            success!("Changes stashed successfully");
        }

        Ok(())
    }
}
