//! Stash list 命令
//!
//! 以表格形式列出所有 stash 条目。

use color_eyre::Result;
use prompt::{info, success, TableBuilder, TableStyle};

use crate::registry;

/// Stash list 命令
pub struct StashListCommand;

impl StashListCommand {
    /// 执行 stash list 命令
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let repo = registry::get_git_repository();

        let entries =
            repo.stash_list().map_err(|e| format!("Failed to list stash entries: {}", e))?;

        if entries.is_empty() {
            info!("No stash entries found");
            return Ok(());
        }

        // 构建表格
        let mut table = TableBuilder::new(vec!["#", "Message", "Branch", "Created"]);

        for entry in &entries {
            let timestamp_str = entry
                .timestamp
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "N/A".to_string());

            table = table.add_row(vec![
                format!("stash@{{{}}}", entry.index),
                entry.message.clone(),
                entry.branch.clone(),
                timestamp_str,
            ]);
        }

        // 显示表格
        table
            .with_title("Stash Entries")
            .with_style(TableStyle::Modern)
            .display()
            .map_err(|e| format!("Failed to display table: {}", e))?;

        success!("Found {} stash entries", entries.len());

        Ok(())
    }
}
