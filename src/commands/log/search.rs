use crate::jira::logs::JiraLogs;
use crate::{log_break, log_debug, log_message, log_success, log_warning};
use anyhow::{Context, Result};
use dialoguer::Input;

/// 搜索关键词命令
pub struct SearchCommand;

impl SearchCommand {
    /// 搜索关键词
    pub fn search(jira_id: Option<String>, search_term: Option<String>) -> Result<()> {
        // 1. 获取 JIRA ID（从参数或交互式输入）
        let jira_id = if let Some(id) = jira_id {
            id
        } else {
            Input::<String>::new()
                .with_prompt("Enter Jira ticket ID (e.g., PROJ-123)")
                .interact()
                .context("Failed to read Jira ticket ID")?
        };

        // 2. 创建 JiraLogs 实例并确保日志文件存在
        let logs = JiraLogs::new().context("Failed to initialize JiraLogs")?;
        logs.ensure_log_file_exists(&jira_id)
            .context("Failed to ensure log file exists")?;

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
        log_debug!("Searching for: '{}'...", term);

        // 同时搜索两个文件
        let (api_results, flutter_api_results) = logs
            .search_keyword_both_files(&jira_id, &term)
            .unwrap_or_else(|_| (Vec::new(), Vec::new()));

        let total_count = api_results.len() + flutter_api_results.len();

        if total_count == 0 {
            log_warning!("No matches found for '{}'", term);
            return Ok(());
        }

        log_break!();
        log_success!("Found {} matches:", total_count);
        log_break!();

        // 显示 api.log 的搜索结果
        if !api_results.is_empty() {
            log_break!('=', 40, "api.log");
            log_break!();
            for entry in api_results {
                if let Some(id) = entry.id {
                    if let Some(url) = entry.url {
                        log_message!("URL: {}, ID: {}", url, id);
                    } else {
                        log_debug!("ID: {} (URL not found)", id);
                    }
                }
            }
            log_break!();
            log_break!();
        }

        // 显示 flutter-api.log 的搜索结果
        if !flutter_api_results.is_empty() {
            log_break!('=', 40, "flutter-api.log");
            log_break!();
            for entry in flutter_api_results {
                if let Some(id) = entry.id {
                    if let Some(url) = entry.url {
                        log_message!("URL: {}, ID: {}", url, id);
                    } else {
                        log_debug!("ID: {} (URL not found)", id);
                    }
                }
            }
        }

        Ok(())
    }
}
