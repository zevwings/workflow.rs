use crate::{log_info, log_success, log_warning, Logs};
use anyhow::{Context, Result};
use std::path::Path;

/// 日志搜索命令
pub struct LogsSearchCommand;

impl LogsSearchCommand {
    /// 在日志文件中搜索关键词
    pub fn search(log_file: &Path, search_term: &str) -> Result<()> {
        log_success!("Searching for: '{}'...", search_term);

        let results =
            Logs::search_keyword(log_file, search_term).context("Failed to search log file")?;

        if results.is_empty() {
            log_warning!("No matches found for '{}'", search_term);
            return Ok(());
        }

        log_success!("\nFound {} matches:\n", results.len());

        for entry in results {
            if let Some(id) = entry.id {
                if let Some(url) = entry.url {
                    log_info!("URL: {}, ID: {}", url, id);
                } else {
                    log_info!("ID: {} (URL not found)", id);
                }
            }
        }

        Ok(())
    }
}
