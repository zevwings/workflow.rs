use crate::base::dialog::InputDialog;
use crate::base::util::Clipboard;
use crate::jira::logs::JiraLogs;
use crate::{log_debug, log_error, log_success};
use anyhow::{Context, Result};

/// 查找请求 ID 命令
pub struct FindCommand;

impl FindCommand {
    /// 查找请求 ID
    pub fn find_request_id(jira_id: Option<String>, request_id: Option<String>) -> Result<()> {
        // 1. 获取 JIRA ID（从参数或交互式输入）
        let jira_id = if let Some(id) = jira_id {
            id
        } else {
            InputDialog::new("Enter Jira ticket ID (e.g., PROJ-123)")
                .prompt()
                .context("Failed to read Jira ticket ID")?
        };

        // 2. 创建 JiraLogs 实例
        let logs = JiraLogs::new().context("Failed to initialize JiraLogs")?;

        // 3. 获取请求 ID（从参数或交互式输入）
        let req_id = if let Some(id) = request_id {
            id
        } else {
            InputDialog::new("Enter request ID to find")
                .prompt()
                .context("Failed to read request ID")?
        };

        // 4. 提取响应内容
        log_debug!("Searching for request ID: {}...", req_id);

        let response_content = logs
            .extract_response_content(&jira_id, &req_id)
            .map_err(|e| {
                log_error!("Failed to extract response content: {}", e);
                e
            })?;

        // 复制到剪贴板（CLI特定操作）
        Clipboard::copy(&response_content).context("Failed to copy to clipboard")?;
        log_success!("Response content copied to clipboard successfully");

        Ok(())
    }
}
