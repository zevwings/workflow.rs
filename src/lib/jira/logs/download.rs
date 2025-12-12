//! 下载功能相关实现（已迁移到 attachments 模块）
//!
//! 此文件保留作为向后兼容的包装器，实际实现已迁移到 `jira::attachments::JiraAttachmentDownloader`

use crate::jira::attachments::{DownloadResult, JiraAttachmentDownloader, ProgressCallback};
use crate::jira::JiraAttachment;
use color_eyre::Result;

use super::JiraLogs;

impl JiraLogs {
    /// 从 Jira ticket 下载日志附件
    ///
    /// 返回下载结果，包含基础目录路径、成功下载的文件列表和失败的文件列表。
    /// 如果下载失败或没有附件/日志，会自动清理已创建的目录。
    ///
    /// 此方法是对 `JiraAttachmentDownloader` 的包装，保持向后兼容。
    ///
    /// # 参数
    ///
    /// * `attachments` - 可选的附件列表（如果提供，将跳过 API 调用以避免重复请求）
    pub fn download_from_jira(
        &self,
        jira_id: &str,
        log_output_folder_name: Option<&str>,
        download_all_attachments: bool,
        callback: Option<ProgressCallback>,
        max_concurrent: Option<usize>,
        attachments: Option<Vec<JiraAttachment>>,
    ) -> Result<DownloadResult> {
        let output_folder = log_output_folder_name
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.output_folder_name.clone());

        // 使用新的下载器
        let downloader = JiraAttachmentDownloader::new()?;
        downloader.download_from_jira(
            jira_id,
            &self.base_dir,
            &output_folder,
            download_all_attachments,
            callback,
            max_concurrent,
            attachments, // 传递附件列表（如果提供）
        )
    }
}
