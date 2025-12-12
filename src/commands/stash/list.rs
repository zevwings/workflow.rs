//! Stash list command
//!
//! List all stash entries in a table format.

use crate::base::util::{TableBuilder, TableStyle};
use crate::git::GitStash;
use crate::{log_break, log_info, log_message, log_success};
use color_eyre::{eyre::WrapErr, Result};
use tabled::Tabled;

/// Stash 表格行
#[derive(Tabled, Clone)]
struct StashRow {
    #[tabled(rename = "#")]
    index: String,
    #[tabled(rename = "Message")]
    message: String,
    #[tabled(rename = "Branch")]
    branch: String,
    #[tabled(rename = "Created")]
    created: String,
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
        log_break!();
        log_message!("Stash List");

        let entries = GitStash::stash_list().wrap_err("Failed to list stash entries")?;

        if entries.is_empty() {
            log_info!("No stash entries found");
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
        let table = TableBuilder::new(rows)
            .with_title("Stash Entries")
            .with_style(TableStyle::Modern)
            .render();

        log_message!("{}", table);

        // 如果请求显示统计信息
        if show_stat {
            log_break!();
            log_message!("File Change Statistics");

            for entry in &entries {
                let stash_ref = format!("stash@{{{}}}", entry.index);
                if let Ok(stat) = GitStash::stash_show_stat(&stash_ref) {
                    log_info!(
                        "stash@{{{}}}: {} files changed, {} insertions(+), {} deletions(-)",
                        entry.index,
                        stat.files_changed,
                        stat.insertions,
                        stat.deletions
                    );
                }
            }
        }

        log_success!("Found {} stash entries", entries.len());

        Ok(())
    }
}
