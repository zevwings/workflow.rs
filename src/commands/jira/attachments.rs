use crate::base::util::dialog::InputDialog;
use crate::jira::logs::JiraLogs;
use crate::{log_debug, log_info, log_success};
use anyhow::{Context, Result};

/// 下载附件命令
pub struct AttachmentsCommand;

impl AttachmentsCommand {
    /// 下载所有附件
    pub fn download(jira_id: Option<String>) -> Result<()> {
        // 获取 JIRA ID（从参数或交互式输入）
        let jira_id = if let Some(id) = jira_id {
            id
        } else {
            InputDialog::new("Enter Jira ticket ID (e.g., PROJ-123)")
                .prompt()
                .context("Failed to read Jira ticket ID")?
        };

        log_success!("Downloading all attachments for {}...", jira_id);

        log_debug!("Getting attachments for {}...", jira_id);

        // 创建 JiraLogs 实例并执行下载
        let logs = JiraLogs::new().context("Failed to initialize JiraLogs")?;
        let base_dir = logs
            .download_from_jira(&jira_id, None, true)
            .context("Failed to download attachments from Jira")?;

        log_success!("Download completed!\n");
        log_info!("Files located at: {}/downloads", base_dir.display());

        Ok(())
    }
}
