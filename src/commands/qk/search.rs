use crate::jira::logs::JiraLogs;
use crate::{log_break, log_debug, log_message, log_success, log_warning};
use anyhow::{Context, Result};
use dialoguer::Input;

/// 搜索关键词命令
#[allow(dead_code)]
pub struct SearchCommand;

#[allow(dead_code)]
impl SearchCommand {
    /// 搜索关键词
    pub fn search(jira_id: &str, search_term: Option<String>) -> Result<()> {
        // 1. 创建 JiraLogs 实例并确保日志文件存在
        let logs = JiraLogs::new().context("Failed to initialize JiraLogs")?;
        let log_file = logs.ensure_log_file_exists(jira_id)?;

        // 2. 获取搜索词（从参数或交互式输入）
        let term = if let Some(t) = search_term {
            t
        } else {
            Input::<String>::new()
                .with_prompt("Enter search term")
                .interact()
                .context("Failed to read search term")?
        };

        // 3. 调用库函数执行搜索
        log_debug!("Searching for: '{}'...", term);

        // 搜索主日志文件（flutter-api.log）
        let flutter_api_results = logs
            .search_keyword(jira_id, &term)
            .unwrap_or_else(|_| Vec::new());

        // 确定 api.log 文件路径并搜索（如果存在）
        let api_log = log_file
            .parent()
            .map(|p| p.join("api.log"))
            .ok_or_else(|| anyhow::anyhow!("Failed to get parent directory"))?;

        let api_results: Vec<crate::LogEntry> = Vec::new();
        if api_log.exists() {
            // 对于 api.log，我们需要从路径中提取 jira_id
            // 或者创建一个新的 JiraLogs 实例来搜索这个文件
            // 由于 search_keyword 现在需要 jira_id，我们需要找到对应的 jira_id
            // 暂时跳过 api.log 的搜索，或者需要添加一个接受路径的方法
            // TODO: 需要支持直接搜索指定路径的日志文件
        }

        let total_count = api_results.len() + flutter_api_results.len();

        if total_count == 0 {
            log_warning!("No matches found for '{}'", term);
            return Ok(());
        }

        log_break!();
        log_success!("Found {} matches:", total_count);
        log_break!();

        // 显示 api.log 的结果
        let has_api_results = !api_results.is_empty();
        if has_api_results {
            log_break!('=', 40, "api.log");
            for entry in api_results {
                if let Some(id) = entry.id {
                    if let Some(url) = entry.url {
                        log_message!("URL: {}, ID: {}", url, id);
                    } else {
                        log_debug!("ID: {} (URL not found)", id);
                    }
                }
            }
        }

        // 显示 flutter-api.log 的结果
        if !flutter_api_results.is_empty() {
            if has_api_results {
                log_break!(); // 添加空行分隔
            }
            log_break!('=', 40, "flutter-api.log");
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
