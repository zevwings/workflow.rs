use crate::core::constants::errors;
use crate::core::prompt::{TableBuilder, TableStyle};
use crate::services::jira::logs::JiraLogs;
use crate::services::jira::logs::SearchResultRow;
use crate::{br, debug, info, success, warning};
use color_eyre::{eyre::WrapErr, Result};

/// 搜索关键词命令
pub struct SearchCommand;

impl SearchCommand {
    /// 搜索关键词
    pub fn search(jira_id: Option<String>, search_term: Option<String>) -> Result<()> {
        // 1. 获取 JIRA ID（从参数或交互式输入）
        let jira_id = if let Some(id) = jira_id {
            id
        } else {
            crate::input!("Enter Jira ticket ID (e.g., PROJ-123)")
                .prompt()
                .wrap_err(errors::client::INPUT_READ_JIRA_TICKET_ID_FAILED)?
        };

        // 2. 创建 JiraLogs 实例并确保日志文件存在
        let logs = JiraLogs::new().wrap_err("Failed to initialize JiraLogs")?;
        logs.ensure_log_file_exists(&jira_id)
            .wrap_err("Failed to ensure log file exists")?;

        // 3. 获取搜索词（从参数或交互式输入）
        let term = if let Some(t) = search_term {
            t
        } else {
            crate::input!("Enter search term")
                .prompt()
                .wrap_err("Failed to read search term")?
        };

        // 4. 调用库函数执行搜索
        debug!("Searching for: '{}'...", term);

        // 同时搜索两个文件
        let (api_results, flutter_api_results) = logs
            .search_keyword_both_files(&jira_id, &term)
            .unwrap_or_else(|_| (Vec::new(), Vec::new()));

        let total_count = api_results.len() + flutter_api_results.len();

        if total_count == 0 {
            warning!("No matches found for '{}'", term);
            return Ok(());
        }

        br!();
        success!("Found {} matches:", total_count);
        br!();

        // 构建表格数据
        let mut rows: Vec<SearchResultRow> = Vec::new();

        // 添加 api.log 的搜索结果
        for entry in api_results {
            if let Some(id) = entry.id {
                rows.push(SearchResultRow {
                    source: "api.log".to_string(),
                    id: id.clone(),
                    url: entry.url.unwrap_or_else(|| "-".to_string()),
                });
            }
        }

        // 添加 flutter-api.log 的搜索结果
        for entry in flutter_api_results {
            if let Some(id) = entry.id {
                rows.push(SearchResultRow {
                    source: "flutter-api.log".to_string(),
                    id: id.clone(),
                    url: entry.url.unwrap_or_else(|| "-".to_string()),
                });
            }
        }

        // 使用表格显示所有结果
        if !rows.is_empty() {
            info!(
                "{}",
                TableBuilder::from_tabled(rows)
                    .with_title("Search Results")
                    .with_style(TableStyle::Modern)
                    .render()
            );
        }

        Ok(())
    }
}
