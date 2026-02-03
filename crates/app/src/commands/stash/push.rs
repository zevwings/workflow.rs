//! Stash push 命令
//!
//! 保存当前工作目录的更改到 stash。

use color_eyre::Result;
use prompt::{info, input, success, warning};

use crate::registry;

/// Stash push 命令
pub struct StashPushCommand;

impl StashPushCommand {
    /// 执行 stash push 命令
    ///
    /// 保存当前工作目录和暂存区的未提交更改到 stash。
    /// 提示用户输入可选的消息来标识 stash 条目。
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let repo = registry::get_git_repository();

        // 检查是否有未提交的更改
        let status = repo
            .get_working_tree_status()
            .map_err(|e| format!("Failed to check working directory status: {}", e))?;

        if status.is_clean() {
            warning!("No changes to stash. Working tree is clean.");
            return Ok(());
        }

        // 提示用户输入 stash 消息（可选）
        let message = input!("Stash message (optional, press Enter to skip)")
            .default("")
            .prompt()
            .map_err(|e| format!("Failed to get stash message: {}", e))?;

        // 执行 stash push
        let stash_message = if message.trim().is_empty() {
            None
        } else {
            Some(message.trim())
        };

        info!("Stashing changes...");

        repo.stash_push(stash_message)
            .map_err(|e| format!("Failed to stash changes: {}", e))?;

        if let Some(msg) = stash_message {
            success!("Changes stashed with message: {}", msg);
        } else {
            success!("Changes stashed successfully");
        }

        Ok(())
    }
}
