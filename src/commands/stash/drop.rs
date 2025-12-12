//! Stash drop command
//!
//! Delete one or more stash entries.

use crate::base::dialog::{ConfirmDialog, MultiSelectDialog};
use crate::git::GitStash;
use crate::{log_break, log_info, log_message, log_success};
use anyhow::{Context, Result};

/// Stash drop command
pub struct StashDropCommand;

impl StashDropCommand {
    /// Execute the stash drop command
    pub fn execute() -> Result<()> {
        log_break!();
        log_message!("Stash Drop");

        // 获取所有 stash 条目
        let entries = GitStash::stash_list().context("Failed to list stash entries")?;

        if entries.is_empty() {
            log_info!("No stash entries available");
            return Ok(());
        }

        // 构建选项列表，格式：stash@{n}: <message> (On <branch>)
        let options: Vec<String> = entries
            .iter()
            .map(|entry| {
                format!(
                    "stash@{{{}}}: {} (On {})",
                    entry.index, entry.message, entry.branch
                )
            })
            .collect();

        // 多选列表
        let selected = MultiSelectDialog::new("Select stash entries to delete", options)
            .prompt()
            .context("Failed to select stash entries")?;

        if selected.is_empty() {
            log_info!("No stash entries selected");
            return Ok(());
        }

        // 从选中的字符串中提取 stash 引用
        let stash_refs: Vec<String> = selected
            .iter()
            .filter_map(|s| s.split(':').next().map(|r| r.trim().to_string()))
            .collect();

        // 显示将要删除的 stash 信息
        log_break!();
        log_message!("Stashes to be deleted:");
        for stash_ref in &stash_refs {
            if let Some(entry) =
                entries.iter().find(|e| format!("stash@{{{}}}", e.index) == *stash_ref)
            {
                log_info!("  {}: {} (On {})", stash_ref, entry.message, entry.branch);
            }
        }

        // 确认删除
        let confirmed = ConfirmDialog::new(format!(
            "Are you sure you want to delete {} stash entry/entries?",
            stash_refs.len()
        ))
        .with_default(false)
        .prompt()
        .context("Failed to get user confirmation")?;

        if !confirmed {
            log_info!("Operation cancelled");
            return Ok(());
        }

        // 删除选中的 stash（从最新的开始删除，避免索引变化）
        // 注意：需要按索引从大到小排序，因为删除后索引会变化
        let mut indices: Vec<usize> = stash_refs
            .iter()
            .filter_map(|ref_str| {
                // 从 "stash@{n}" 中提取 n
                ref_str
                    .strip_prefix("stash@{")
                    .and_then(|s| s.strip_suffix("}"))
                    .and_then(|s| s.parse::<usize>().ok())
            })
            .collect();

        // 按索引从大到小排序，这样删除时不会影响其他索引
        indices.sort_by(|a, b| b.cmp(a));

        let mut deleted_count = 0;
        for index in indices {
            let stash_ref = format!("stash@{{{}}}", index);
            match GitStash::stash_drop(Some(&stash_ref)) {
                Ok(_) => {
                    log_success!("Stash {} deleted successfully", stash_ref);
                    deleted_count += 1;
                }
                Err(e) => {
                    log_info!("Failed to delete stash {}: {}", stash_ref, e);
                }
            }
        }

        if deleted_count > 0 {
            log_success!("Successfully deleted {} stash entry/entries", deleted_count);
        }

        Ok(())
    }
}
