use crate::{log_info, log_success, log_warning, Logs};
use anyhow::{Context, Result};
use dialoguer::Input;

/// 搜索关键词命令
#[allow(dead_code)]
pub struct SearchCommand;

impl SearchCommand {
    /// 搜索关键词
    #[allow(dead_code)]
    pub fn search(jira_id: &str, search_term: Option<String>) -> Result<()> {
        // 1. 获取日志文件路径
        let log_file = Logs::get_log_file_path(jira_id)?;

        // 2. 检查日志文件是否存在
        if !log_file.exists() {
            anyhow::bail!(
                "❌ Log file not found at: {:?}\n💡 Try downloading logs first with: workflow qk {} download",
                log_file, jira_id
            );
        }

        // 3. 获取搜索词（从参数或交互式输入）
        let term = if let Some(t) = search_term {
            t
        } else {
            Input::<String>::new()
                .with_prompt("Enter search term")
                .interact()
                .context("Failed to read search term")?
        };

        // 4. 调用库函数执行搜索
        log_success!("Searching for: '{}'...", term);

        let results =
            Logs::search_keyword(&log_file, &term).context("Failed to search log file")?;

        if results.is_empty() {
            log_warning!("No matches found for '{}'", term);
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
