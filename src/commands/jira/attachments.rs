use crate::base::indicator::{Progress, Spinner};
use crate::jira::logs::{JiraLogs, ProgressCallback};
use crate::jira::Jira;
use crate::{log_break, log_info, log_success};
use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};

use super::helpers::get_jira_id;

/// 下载附件命令
pub struct AttachmentsCommand;

impl AttachmentsCommand {
    /// 下载所有附件
    pub fn download(jira_id: Option<String>) -> Result<()> {
        // 获取 JIRA ID（从参数或交互式输入）
        let jira_id = get_jira_id(jira_id, None)?;

        // 先获取附件列表以确定总数（使用 Spinner 显示加载状态）
        let attachments = Spinner::with(
            format!("Getting attachments info for {}...", jira_id),
            || Jira::get_attachments(&jira_id).context("Failed to get attachments from Jira"),
        )?;
        let total_files = attachments.len() as u64;

        if total_files == 0 {
            anyhow::bail!("No attachments found for {}", jira_id);
        }

        // 显示下载前的提示信息
        log_info!("{} file(s) will be downloaded", total_files);
        log_break!();

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

        // 执行下载（传递已获取的附件列表，避免重复 API 调用）
        let result = logs
            .download_from_jira(
                &jira_id,
                None,
                true,
                Some(callback),
                None,
                Some(attachments),
            )
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

        // 显示下载的文件列表
        if !result.downloaded_files.is_empty() {
            log_break!();
            log_info!("Downloaded {} file(s):", result.downloaded_files.len());
            for file_path in &result.downloaded_files {
                // 只显示文件名，让输出更简洁
                if let Some(file_name) = file_path.file_name() {
                    log_info!("  ✓ {}", file_name.to_string_lossy());
                } else {
                    log_info!("  ✓ {}", file_path.display());
                }
            }
        }

        log_info!("Files located at: {}/downloads", result.base_dir.display());

        Ok(())
    }
}
