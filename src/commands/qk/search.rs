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

        // 4. 调用库函数执行搜索（支持在多个日志文件中搜索）
        log_success!("Searching for: '{}'...", term);

        // 确定两个日志文件路径
        let api_log = log_file.parent()
            .map(|p| p.join("api.log"))
            .ok_or_else(|| anyhow::anyhow!("Failed to get parent directory"))?;
        let flutter_api_log = log_file.parent()
            .map(|p| p.join("flutter-api.log"))
            .ok_or_else(|| anyhow::anyhow!("Failed to get parent directory"))?;

        // 分别搜索两个文件
        let mut api_results = Vec::new();
        let mut flutter_api_results = Vec::new();

        if api_log.exists() {
            if let Ok(results) = Logs::search_keyword(&api_log, &term) {
                api_results = results;
            }
        }

        if flutter_api_log.exists() {
            if let Ok(results) = Logs::search_keyword(&flutter_api_log, &term) {
                flutter_api_results = results;
            }
        }

        let total_count = api_results.len() + flutter_api_results.len();

        if total_count == 0 {
            log_warning!("No matches found for '{}'", term);
            return Ok(());
        }

        log_success!("\nFound {} matches:\n", total_count);

        // 显示 api.log 的结果
        let has_api_results = !api_results.is_empty();
        if has_api_results {
            log_info!("===========  api.log ===========");
            for entry in api_results {
                if let Some(id) = entry.id {
                    if let Some(url) = entry.url {
                        log_info!("URL: {}, ID: {}", url, id);
                    } else {
                        log_info!("ID: {} (URL not found)", id);
                    }
                }
            }
        }

        // 显示 flutter-api.log 的结果
        if !flutter_api_results.is_empty() {
            if has_api_results {
                log_info!(""); // 添加空行分隔
            }
            log_info!("===========  flutter-api.log ===========");
            for entry in flutter_api_results {
                if let Some(id) = entry.id {
                    if let Some(url) = entry.url {
                        log_info!("URL: {}, ID: {}", url, id);
                    } else {
                        log_info!("ID: {} (URL not found)", id);
                    }
                }
            }
        }

        Ok(())
    }
}
