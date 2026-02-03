//! Stash drop 命令
//!
//! 删除一个或多个 stash 条目。

use color_eyre::Result;
use prompt::{confirm, info, multiselect, success, warning};

use crate::registry;

/// Stash drop 命令
pub struct StashDropCommand;

impl StashDropCommand {
    /// 执行 stash drop 命令
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let repo = registry::get_git_repository();

        // 获取所有 stash 条目
        let entries = repo
            .stash_list()
            .map_err(|e| format!("Failed to list stash entries: {}", e))?;

        if entries.is_empty() {
            info!("No stash entries available");
            return Ok(());
        }

        // 构建选项列表
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
        let selected = multiselect!("Select stash entries to delete", options)
            .prompt()
            .map_err(|e| format!("Failed to select stash entries: {}", e))?;

        if selected.is_empty() {
            info!("No stash entries selected");
            return Ok(());
        }

        // 从选中的字符串中提取 stash 索引
        let mut indices: Vec<usize> = selected
            .iter()
            .filter_map(|s: &String| {
                s.split(':')
                    .next()
                    .and_then(|r: &str| r.trim().strip_prefix("stash@{"))
                    .and_then(|r: &str| r.strip_suffix("}"))
                    .and_then(|r: &str| r.parse::<usize>().ok())
            })
            .collect();

        // 显示将要删除的 stash 信息
        info!("Stashes to be deleted:");
        for index in &indices {
            if let Some(entry) = entries.iter().find(|e| e.index == *index) {
                info!(
                    "  stash@{{{}}}: {} (On {})",
                    entry.index, entry.message, entry.branch
                );
            }
        }

        // 确认删除
        let confirmed = confirm!(
            "Are you sure you want to delete {} stash entry/entries?",
            indices.len()
        )
        .default(false)
        .prompt()
        .map_err(|e| format!("Failed to get user confirmation: {}", e))?;

        if !confirmed {
            info!("Operation cancelled");
            return Ok(());
        }

        // 按索引从大到小排序，这样删除时不会影响其他索引
        indices.sort_by(|a: &usize, b: &usize| b.cmp(a));

        let mut deleted_count = 0;
        for index in indices {
            match repo.stash_drop(index) {
                Ok(_) => {
                    success!("Stash stash@{{{}}} deleted successfully", index);
                    deleted_count += 1;
                }
                Err(e) => {
                    warning!("Failed to delete stash stash@{{{}}}: {}", index, e);
                }
            }
        }

        if deleted_count > 0 {
            success!("Successfully deleted {} stash entry/entries", deleted_count);
        }

        Ok(())
    }
}
