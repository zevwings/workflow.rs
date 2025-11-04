use crate::{log_info, log_success, Logs};
use crate::settings::Settings;
use anyhow::{Context, Result};

/// 下载日志命令
pub struct DownloadCommand;

impl DownloadCommand {
    /// 下载日志
    pub fn download(jira_id: &str) -> Result<()> {
        log_success!("Downloading logs for {}...", jira_id);

        let settings = Settings::get();
        let log_output_folder_name = if !settings.log_output_folder_name.is_empty() {
            Some(settings.log_output_folder_name.as_str())
        } else {
            None
        };

        log_success!("Getting attachments for {}...", jira_id);

        // 调用库函数执行下载
        let base_dir = Logs::download_from_jira(jira_id, log_output_folder_name)
            .context("Failed to download logs from Jira")?;

        log_success!("\nDownload completed!");
        log_info!("Files located at: {:?}", base_dir);

        Ok(())
    }
}

