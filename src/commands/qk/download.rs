use crate::jira::logs::JiraLogs;
use crate::{log_debug, log_info, log_success};
use anyhow::{Context, Result};

/// 下载日志命令
#[allow(dead_code)]
pub struct DownloadCommand;

#[allow(dead_code)]
impl DownloadCommand {
    /// 下载日志
    pub fn download(jira_id: &str, download_all: bool) -> Result<()> {
        if download_all {
            log_success!("Downloading all attachments for {}...", jira_id);
        } else {
            log_success!("Downloading logs for {}...", jira_id);
        }

        log_debug!("Getting attachments for {}...", jira_id);

        // 创建 JiraLogs 实例并执行下载
        let logs = JiraLogs::new().context("Failed to initialize JiraLogs")?;
        let base_dir = logs
            .download_from_jira(jira_id, None, download_all)
            .context("Failed to download attachments from Jira")?;

        log_success!("Download completed!\n");
        log_info!("Files located at: {}/downloads", base_dir.display());

        Ok(())
    }
}
