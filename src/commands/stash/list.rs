//! Stash list command
//!
//! List all stash entries in a table format.

use crate::git::GitStash;
use crate::interactive::{TableBuilder, TableStyle, Tabled};
use crate::{br, info, success};
use color_eyre::{eyre::WrapErr, Result};

/// Stash 表格行
#[derive(Clone)]
struct StashRow {
    index: String,
    message: String,
    branch: String,
    created: String,
}

impl Tabled for StashRow {
    fn headers() -> Vec<String> {
        vec![
            "#".to_string(),
            "Message".to_string(),
            "Branch".to_string(),
            "Created".to_string(),
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.index.clone(),
            self.message.clone(),
            self.branch.clone(),
            self.created.clone(),
        ]
    }
}

/// Stash list command
pub struct StashListCommand;

impl StashListCommand {
    /// Execute the stash list command
    ///
    /// # Arguments
    ///
    /// * `show_stat` - Whether to show file change statistics
    pub fn execute(show_stat: bool) -> Result<()> {
        br!();
        info!("Stash List");

        let entries = GitStash::stash_list().wrap_err("Failed to list stash entries")?;

        if entries.is_empty() {
            info!("No stash entries found");
            return Ok(());
        }

        // 构建表格数据
        let rows: Vec<StashRow> = entries
            .iter()
            .map(|entry| {
                let timestamp_str = entry
                    .timestamp
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "N/A".to_string());

                StashRow {
                    index: format!("stash@{{{}}}", entry.index),
                    message: entry.message.clone(),
                    branch: entry.branch.clone(),
                    created: timestamp_str,
                }
            })
            .collect();

        // 显示表格
        let table = TableBuilder::from_tabled(rows)
            .with_title("Stash Entries")
            .with_style(TableStyle::Modern)
            .render();

        info!("{}", table);

        // 如果请求显示统计信息
        if show_stat {
            br!();
            info!("File Change Statistics");

            for entry in &entries {
                let stash_ref = format!("stash@{{{}}}", entry.index);
                if let Ok(stat) = GitStash::stash_show_stat(&stash_ref) {
                    info!(
                        "stash@{{{}}}: {} files changed, {} insertions(+), {} deletions(-)",
                        entry.index, stat.files_changed, stat.insertions, stat.deletions
                    );
                }
            }
        }

        success!("Found {} stash entries", entries.len());

        Ok(())
    }
}
