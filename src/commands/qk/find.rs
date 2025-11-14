use crate::{log_debug, log_error, log_success, Clipboard, Logs};
use anyhow::{Context, Result};
use dialoguer::Input;

/// 查找请求 ID 命令
#[allow(dead_code)]
pub struct FindCommand;

#[allow(dead_code)]
impl FindCommand {
    /// 查找请求 ID
    pub fn find_request_id(jira_id: &str, request_id: Option<String>) -> Result<()> {
        // 1. 确保日志文件存在
        let log_file = Logs::ensure_log_file_exists(jira_id)?;

        // 2. 获取请求 ID（从参数或交互式输入）
        let req_id = if let Some(id) = request_id {
            id
        } else {
            Input::<String>::new()
                .with_prompt("Enter request ID to find")
                .interact()
                .context("Failed to read request ID")?
        };

        // 3. 提取响应内容
        log_debug!("Searching for request ID: {}...", req_id);

        let response_content = Logs::extract_response_content(&log_file, &req_id).map_err(|e| {
            log_error!("Failed to extract response content: {}", e);
            e
        })?;

        // 复制到剪贴板（CLI特定操作）
        Clipboard::copy(&response_content).context("Failed to copy to clipboard")?;
        log_success!("Response content copied to clipboard successfully");

        Ok(())
    }
}
