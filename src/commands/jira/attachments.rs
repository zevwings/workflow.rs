use crate::base::util::dialog::InputDialog;
use crate::jira::logs::{JiraLogs, ProgressCallback};
use crate::{log_break, log_info, log_success};
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

        // 创建 JiraLogs 实例
        let logs = JiraLogs::new().context("Failed to initialize JiraLogs")?;

        // 创建回调函数，将进度消息输出到控制台
        let callback: ProgressCallback = Box::new(|msg| {
            if !msg.is_empty() {
                log_info!("{}", msg);
            } else {
                log_break!();
            }
        });

        // 执行下载
        let result = logs
            .download_from_jira(&jira_id, None, true, Some(callback))
            .context("Failed to download attachments from Jira")?;

        // 显示下载结果
        if !result.failed_files.is_empty() {
            log_break!();
            log_info!(
                "  Warning: {} attachment(s) failed to download:",
                result.failed_files.len()
            );
            for (filename, error) in &result.failed_files {
                log_info!("  - {}: {}", filename, error);
            }
        }

        log_success!("Download completed!");
        log_info!("Files located at: {}/downloads", result.base_dir.display());

        Ok(())
    }
}
