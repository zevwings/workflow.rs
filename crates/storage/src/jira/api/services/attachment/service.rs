//! 附件服务核心实现
//!
//! 整合所有组件，提供完整的附件下载功能。

use std::{path::Path, sync::Arc};

use client::JiraConfigContext;
use domain::{AttachmentDownloadResult, JiraError, JiraIssue, ProgressCallback};

use crate::jira::api::services::attachment::{
    directory::DirectoryManager, downloader::ConcurrentDownloader, entity::UrlResolver,
};
use crate::jira::api::services::IssueService;

/// 附件服务 trait
///
/// 定义附件下载和清理的接口。
pub trait AttachmentService: Send + Sync {
    /// 下载附件
    ///
    /// # 参数
    ///
    /// * `issue_id` - Jira ticket ID
    /// * `base_dir` - 基础目录路径
    /// * `on_progress` - 进度回调函数（可选）
    fn download_attachments(
        &self,
        issue_id: &str,
        base_dir: &Path,
        on_progress: Option<ProgressCallback>,
    ) -> Result<AttachmentDownloadResult, JiraError>;

    /// 使用已获取的 Issue 数据下载附件（避免重复 API 调用）
    ///
    /// # 参数
    ///
    /// * `issue` - 已获取的 Jira Issue 信息
    /// * `base_dir` - 基础目录路径
    /// * `on_progress` - 进度回调函数（可选）
    fn download_attachments_with_issue(
        &self,
        issue: &JiraIssue,
        base_dir: &Path,
        on_progress: Option<ProgressCallback>,
    ) -> Result<AttachmentDownloadResult, JiraError>;

    /// 清理附件目录
    ///
    /// # 参数
    ///
    /// * `jira_id` - JIRA ID（如 "PROJ-123"）。如果为 `None`，清理所有附件目录
    fn clean_attachments(&self, jira_id: Option<&str>) -> Result<(), JiraError>;
}

/// 附件服务实现
pub struct AttachmentServiceImpl {
    issue_service: Arc<dyn IssueService>,
    config_context: Arc<dyn JiraConfigContext>,
    downloader: ConcurrentDownloader,
}

impl AttachmentServiceImpl {
    /// 创建新的附件服务实例
    pub fn new(
        issue_service: Arc<dyn IssueService>,
        config_context: Arc<dyn JiraConfigContext>,
    ) -> Self {
        let downloader = ConcurrentDownloader::new(Arc::clone(&config_context));
        Self {
            issue_service,
            config_context,
            downloader,
        }
    }
}

impl AttachmentService for AttachmentServiceImpl {
    fn download_attachments(
        &self,
        issue_id: &str,
        base_dir: &Path,
        on_progress: Option<ProgressCallback>,
    ) -> Result<AttachmentDownloadResult, JiraError> {
        // 1. 准备下载目录
        let download_dir = DirectoryManager::prepare_directory(base_dir, issue_id)?;

        // 初始化结果
        let mut result = AttachmentDownloadResult {
            base_dir: download_dir.clone(),
            downloaded_files: Vec::new(),
            failed_files: Vec::new(),
        };

        // 使用闭包进行错误处理，失败时清理目录
        let download_result = (|| -> Result<(), JiraError> {
            // 2. 获取 Issue 信息（包含附件列表）
            let issue = self.issue_service.get_issue_info(issue_id)?;

            // 3. 提取附件列表
            let attachments = issue.fields.attachment.clone().unwrap_or_default();

            if attachments.is_empty() {
                return Err(JiraError::ApiError(format!(
                    "No attachments found for {}",
                    issue_id
                )));
            }

            // 4. 创建 URL 解析器（提供多重 URL 重试）
            let url_resolver = UrlResolver::new(&issue, Arc::clone(&self.config_context));

            // 5. 准备下载任务
            let tasks: Vec<_> = attachments
                .iter()
                .map(|attachment| {
                    let file_path = download_dir.join(&attachment.filename);
                    let urls = url_resolver.get_download_urls(attachment);
                    (
                        attachment.filename.clone(),
                        attachment.clone(),
                        file_path,
                        urls,
                    )
                })
                .collect();

            // 6. 并发下载
            let max_concurrent = 5; // 默认并发数

            // 根据是否有进度回调，分别处理
            let (downloaded, failed) = match on_progress {
                Some(progress_fn) => {
                    // 创建包装回调：将 domain 的 Box<dyn Fn()> 转换为 downloader 的 Box<dyn Fn(&str)>
                    let wrapper: Box<dyn Fn(&str) + Send + Sync> = Box::new(move |_: &str| {
                        // 调用进度回调
                        (*progress_fn)();
                    });
                    self.downloader.download_concurrent(tasks, max_concurrent, Some(&wrapper))
                }
                None => self.downloader.download_concurrent(tasks, max_concurrent, None),
            };

            result.downloaded_files.extend(downloaded);
            result.failed_files.extend(failed);

            Ok(())
        })();

        // 失败时清理目录
        if download_result.is_err() && download_dir.exists() {
            let _ = DirectoryManager::cleanup_on_failure(&download_dir);
            return download_result.map(|_| result);
        }

        download_result.map(|_| result)
    }

    fn download_attachments_with_issue(
        &self,
        issue: &JiraIssue,
        base_dir: &Path,
        on_progress: Option<ProgressCallback>,
    ) -> Result<AttachmentDownloadResult, JiraError> {
        let issue_id = &issue.key;
        let download_dir = DirectoryManager::prepare_directory(base_dir, issue_id)?;

        let mut result = AttachmentDownloadResult {
            base_dir: download_dir.clone(),
            downloaded_files: Vec::new(),
            failed_files: Vec::new(),
        };

        let attachments = issue.fields.attachment.clone().unwrap_or_default();

        if attachments.is_empty() {
            return Err(JiraError::ApiError(format!(
                "No attachments found for {}",
                issue_id
            )));
        }

        let url_resolver = UrlResolver::new(issue, Arc::clone(&self.config_context));

        let tasks: Vec<_> = attachments
            .iter()
            .map(|attachment| {
                let file_path = download_dir.join(&attachment.filename);
                let urls = url_resolver.get_download_urls(attachment);
                (
                    attachment.filename.clone(),
                    attachment.clone(),
                    file_path,
                    urls,
                )
            })
            .collect();

        let max_concurrent = 5;
        let (downloaded, failed) = match on_progress {
            Some(progress_fn) => {
                let wrapper: Box<dyn Fn(&str) + Send + Sync> =
                    Box::new(move |_: &str| (*progress_fn)());
                self.downloader.download_concurrent(tasks, max_concurrent, Some(&wrapper))
            }
            None => self.downloader.download_concurrent(tasks, max_concurrent, None),
        };

        result.downloaded_files.extend(downloaded);
        result.failed_files.extend(failed);

        Ok(result)
    }

    fn clean_attachments(&self, jira_id: Option<&str>) -> Result<(), JiraError> {
        // 获取下载基础目录
        let base_dir = self
            .config_context
            .get_download_dir()
            .map_err(|e| JiraError::ConfigError(format!("Failed to get download dir: {}", e)))?;

        let dir = if let Some(id) = jira_id {
            base_dir.join(id)
        } else {
            base_dir
        };

        if !dir.exists() {
            // 目录不存在，直接返回成功
            return Ok(());
        }

        std::fs::remove_dir_all(&dir).map_err(|e| {
            JiraError::IoError(format!("Failed to delete directory {:?}: {}", dir, e))
        })?;

        Ok(())
    }
}
