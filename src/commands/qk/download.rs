use crate::base::settings::Settings;
use crate::{log_debug, log_info, log_success, Logs};
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

        let settings = Settings::get();
        let log_output_folder_name = if !settings.log.output_folder_name.is_empty() {
            Some(settings.log.output_folder_name.as_str())
        } else {
            None
        };

        log_debug!("Getting attachments for {}...", jira_id);

        // 调用库函数执行下载
        let base_dir = Logs::download_from_jira(jira_id, log_output_folder_name, download_all)
            .context("Failed to download attachments from Jira")?;

        log_success!("Download completed!\n");
        log_info!("Files located at: {}/downloads", base_dir.display());

        Ok(())
    }
}
