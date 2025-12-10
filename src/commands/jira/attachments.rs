use crate::base::dialog::InputDialog;
use crate::base::indicator::Progress;
use crate::jira::logs::{JiraLogs, ProgressCallback};
use crate::jira::Jira;
use crate::{log_break, log_info, log_success};
use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};

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

        // 先获取附件列表以确定总数
        let attachments =
            Jira::get_attachments(&jira_id).context("Failed to get attachments from Jira")?;
        let total_files = attachments.len() as u64;

        if total_files == 0 {
            anyhow::bail!("No attachments found for {}", jira_id);
        }

        // 创建 Progress Bar
        let progress = Arc::new(Mutex::new(Progress::new(
            total_files,
            "Downloading attachments...",
        )));
        let progress_clone = progress.clone();

        // 创建回调函数，更新进度条
        let callback: ProgressCallback = Box::new(move |msg| {
            if !msg.is_empty() {
                if msg.starts_with("Downloaded:") || msg.contains("Downloaded:") {
                    if let Ok(pb) = progress_clone.lock() {
                        pb.inc(1);
                    }
                } else if msg.starts_with("Failed to download:") {
                    // 失败的文件也计入进度
                    if let Ok(pb) = progress_clone.lock() {
                        pb.inc(1);
                    }
                }
            }
        });

        // 创建 JiraLogs 实例
        let logs = JiraLogs::new().context("Failed to initialize JiraLogs")?;

        // 执行下载
        let result = logs
            .download_from_jira(&jira_id, None, true, Some(callback))
            .context("Failed to download attachments from Jira")?;

        // 完成进度条
        if let Ok(pb) = progress.lock() {
            pb.finish_ref();
        }

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
