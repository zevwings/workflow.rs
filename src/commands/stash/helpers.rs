//! Stash command helpers
//!
//! Helper functions for stash commands, including interactive selection.

use crate::base::dialog::SelectDialog;
use crate::git::{GitStash, StashEntry};
use anyhow::{Context, Result};

/// 交互式选择 stash 条目
///
/// 显示所有 stash 条目供用户选择，返回选中的 stash 引用（如 "stash@{0}"）。
///
/// # 返回
///
/// 返回选中的 stash 引用，格式为 "stash@{n}"。
pub fn select_stash_interactively() -> Result<String> {
    let entries = GitStash::stash_list().context("Failed to list stash entries")?;

    if entries.is_empty() {
        anyhow::bail!("No stash entries available");
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

    let selected = SelectDialog::new("Select a stash entry", options)
        .with_default(0)
        .prompt()
        .context("Failed to select stash entry")?;

    // 从选中的字符串中提取 stash 引用
    // 格式：stash@{n}: <message> (On <branch>)
    if let Some(stash_ref) = selected.split(':').next() {
        Ok(stash_ref.trim().to_string())
    } else {
        anyhow::bail!("Failed to parse selected stash reference")
    }
}

/// 格式化 stash 条目用于显示
///
/// 返回格式化的字符串，用于在表格或列表中显示。
pub fn format_stash_entry(entry: &StashEntry, include_stat: bool) -> String {
    let timestamp_str = entry
        .timestamp
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "N/A".to_string());

    if include_stat {
        // 如果有统计信息，可以在这里添加
        format!(
            "stash@{{{}}}: {} (On {}, {})",
            entry.index, entry.message, entry.branch, timestamp_str
        )
    } else {
        format!(
            "stash@{{{}}}: {} (On {}, {})",
            entry.index, entry.message, entry.branch, timestamp_str
        )
    }
}
