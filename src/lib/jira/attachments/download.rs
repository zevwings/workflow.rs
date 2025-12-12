//! Jira 附件下载实现

use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use crate::base::concurrent::TaskResult;
use crate::{trace_debug, Jira, JiraAttachment};

use super::constants::*;
use super::directory::DirectoryManager;
use super::filter::AttachmentFilter;
use super::http_client::AttachmentDownloader;
use super::url_resolver::UrlResolver;
use super::zip::ZipProcessor;

/// 进度回调函数类型
pub type ProgressCallback = Box<dyn Fn(&str) + Send + Sync>;

/// 下载操作结果类型（成功文件列表，失败文件列表）
type DownloadOperationResult = (Vec<PathBuf>, Vec<(String, String)>);

/// 下载结果
#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub base_dir: PathBuf,
    pub downloaded_files: Vec<PathBuf>,
    pub failed_files: Vec<(String, String)>, // (filename, error)
}

/// Jira 附件下载器
///
/// 提供从 Jira 下载附件的功能，包括所有附件和日志附件的下载。
pub struct JiraAttachmentDownloader {
    /// ZIP 处理器
    zip_processor: ZipProcessor,
}

impl JiraAttachmentDownloader {
    /// 创建新的下载器实例
    pub fn new() -> Result<Self> {
        Ok(Self {
            zip_processor: ZipProcessor,
        })
    }

    /// 从 Jira ticket 下载附件
    ///
    /// # 参数
    ///
    /// * `jira_id` - Jira ticket ID
    /// * `base_dir` - 基础目录路径（用于创建下载目录）
    /// * `output_folder_name` - 输出文件夹名称（用于日志解压）
    /// * `download_all_attachments` - 是否下载所有附件（true）或仅日志附件（false）
    /// * `callback` - 进度回调函数
    /// * `max_concurrent` - 最大并发数（可选，默认 5）
    /// * `attachments` - 可选的附件列表（如果提供，将跳过 API 调用）
    ///
    /// # 返回
    ///
    /// 返回下载结果，包含基础目录路径、成功下载的文件列表和失败的文件列表。
    /// 如果下载失败或没有附件/日志，会自动清理已创建的目录。
    pub fn download_from_jira(
        &self,
        jira_id: &str,
        base_dir: &Path,
        output_folder_name: &str,
        download_all_attachments: bool,
        callback: Option<ProgressCallback>,
        max_concurrent: Option<usize>,
        attachments: Option<Vec<JiraAttachment>>,
    ) -> Result<DownloadResult> {
        self.call_callback(callback.as_ref(), "Preparing download directory...");

        // 1. 准备下载目录
        let (download_base_dir, download_dir) =
            DirectoryManager::prepare_download_directory(base_dir, jira_id)?;
        let base_dir_path = download_base_dir.clone();

        // 初始化结果结构
        let mut result = DownloadResult {
            base_dir: download_base_dir.clone(),
            downloaded_files: Vec::new(),
            failed_files: Vec::new(),
        };

        // 使用 Result 的错误处理，在失败时清理目录
        let download_result = (|| -> Result<()> {
            // 2. 获取并过滤附件（如果未提供，则从 API 获取）
            let attachments = if let Some(attachments) = attachments {
                self.call_callback(callback.as_ref(), "Using provided attachments...");
                attachments
            } else {
                self.call_callback(callback.as_ref(), "Fetching attachments...");
                Jira::get_attachments(jira_id).wrap_err("Failed to get attachments from Jira")?
            };

            if attachments.is_empty() {
                color_eyre::eyre::bail!("No attachments found for {}", jira_id);
            }

            // 调试：显示所有附件
            trace_debug!("Found {} attachment(s):", attachments.len());
            for attachment in &attachments {
                trace_debug!("  - {}", attachment.filename);
            }

            let target_attachments = if download_all_attachments {
                attachments
            } else {
                let log_attachments = AttachmentFilter::filter_log_attachments(&attachments);
                if log_attachments.is_empty() {
                    color_eyre::eyre::bail!("No log attachments found for {}", jira_id);
                }
                log_attachments
            };

            // 3. 创建 URL 解析器（如果需要）
            let url_resolver = if !download_all_attachments {
                Some(UrlResolver::from_ticket(jira_id)?)
            } else {
                None
            };

            // 4. 下载附件
            let max_concurrent = max_concurrent.unwrap_or(5).clamp(1, 20); // 默认 5，范围 1-20
            if download_all_attachments {
                self.call_callback(callback.as_ref(), "Downloading all attachments...");
            } else {
                self.call_callback(callback.as_ref(), "Downloading log attachments...");
            }

            let (downloaded, failed) = self.download_attachments(
                &target_attachments,
                &download_dir,
                url_resolver.as_ref(),
                callback.as_ref(),
                max_concurrent,
            )?;
            result.downloaded_files.extend(downloaded);
            result.failed_files.extend(failed);

            self.call_callback(callback.as_ref(), "Processing downloaded logs...");

            // 5. 处理下载的日志（合并分片、解压）
            self.process_downloaded_logs(
                &download_base_dir,
                &download_dir,
                output_folder_name,
                download_all_attachments,
            )?;

            Ok(())
        })();

        // 如果失败，清理已创建的目录
        if download_result.is_err() && base_dir_path.exists() {
            let _ = DirectoryManager::cleanup_on_failure(&base_dir_path);
            return download_result.map(|_| result);
        }

        download_result.map(|_| result)
    }

    /// 尝试下载单个附件（使用多个 URL 重试）
    fn try_download_attachment(
        attachment: &JiraAttachment,
        file_path: &Path,
        urls: &[String],
    ) -> Result<PathBuf, String> {
        for url in urls {
            match AttachmentDownloader::download_file(url, file_path) {
                Ok(()) => return Ok(file_path.to_path_buf()),
                Err(e) => {
                    trace_debug!(
                        "Failed to download {} from {}: {}",
                        attachment.filename,
                        url,
                        e
                    );
                }
            }
        }
        Err(format!(
            "Failed to download {} from all URLs",
            attachment.filename
        ))
    }

    /// 下载附件（使用并发执行器）
    fn download_attachments(
        &self,
        attachments: &[JiraAttachment],
        download_dir: &Path,
        url_resolver: Option<&UrlResolver>,
        callback: Option<&ProgressCallback>,
        max_concurrent: usize,
    ) -> Result<DownloadOperationResult> {
        let max_concurrent = max_concurrent.max(1).min(attachments.len().max(1));

        // 优化：小批量文件使用串行下载，避免线程开销
        if attachments.len() <= 1 {
            return self.download_attachments_sequential(
                attachments,
                download_dir,
                url_resolver,
                callback,
            );
        }

        let download_dir = download_dir.to_path_buf();

        // 准备任务列表
        let mut tasks = Vec::new();
        for attachment in attachments {
            let download_dir = download_dir.clone();
            let filename_for_result = attachment.filename.clone();
            let attachment_clone = attachment.clone();

            // 如果有 URL 解析器，获取所有可能的 URL；否则只使用原始 URL
            let urls = if let Some(resolver) = url_resolver {
                resolver.get_download_urls(&attachment_clone)
            } else {
                vec![attachment_clone.content_url.clone()]
            };

            let task = Box::new(move || -> Result<PathBuf, String> {
                let file_path = download_dir.join(&attachment_clone.filename);
                Self::try_download_attachment(&attachment_clone, &file_path, &urls)
            }) as Box<dyn Fn() -> Result<PathBuf, String> + Send + Sync>;

            tasks.push((filename_for_result, task));
        }

        // 使用自定义执行逻辑，在接收结果时实时调用回调
        // 这样可以确保进度条实时更新，而不是等待所有任务完成
        let max_concurrent = max_concurrent.max(1).min(tasks.len().max(1));

        if tasks.is_empty() {
            return Ok((Vec::new(), Vec::new()));
        }

        // 如果只有一个任务，直接执行
        if tasks.len() == 1 {
            let (name, task) = tasks.into_iter().next().unwrap();
            let result = match task() {
                Ok(value) => {
                    if let Some(cb) = callback {
                        cb(&format!("Downloaded: {}", name));
                    }
                    TaskResult::Success(value)
                }
                Err(err) => {
                    if let Some(cb) = callback {
                        cb(&format!("Failed to download: {} - {}", name, err));
                    }
                    TaskResult::Failure(err)
                }
            };
            let (downloaded, failed) = self.collect_download_results(vec![(name, result)]);
            return Ok((downloaded, failed));
        }

        // 结果通道
        let (tx, rx) = mpsc::channel();

        // 分批处理：将任务分成多个批次，每批最多 max_concurrent 个并行执行
        let mut handles = Vec::new();
        let mut tasks_iter = tasks.into_iter();

        loop {
            let mut chunk = Vec::new();
            for _ in 0..max_concurrent {
                if let Some(task) = tasks_iter.next() {
                    chunk.push(task);
                } else {
                    break;
                }
            }

            if chunk.is_empty() {
                break;
            }

            let tx = tx.clone();

            let handle = thread::spawn(move || {
                for (name, task) in chunk {
                    let result = match task() {
                        Ok(value) => TaskResult::Success(value),
                        Err(err) => TaskResult::Failure(err),
                    };
                    tx.send((name, result)).ok();
                }
            });

            handles.push(handle);
        }

        // 关闭发送端
        drop(tx);

        // 收集结果，并在接收时实时调用回调
        let mut results = Vec::new();
        for (name, result) in rx {
            // 实时调用回调
            if let Some(cb) = callback {
                match &result {
                    TaskResult::Success(_) => {
                        cb(&format!("Downloaded: {}", name));
                    }
                    TaskResult::Failure(err) => {
                        cb(&format!("Failed to download: {} - {}", name, err));
                    }
                }
            }
            results.push((name, result));
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().map_err(|e| eyre!("Thread join error: {:?}", e))?;
        }

        // 收集结果
        let (downloaded, failed) = self.collect_download_results(results);

        self.report_failed_downloads(&failed, callback);
        Ok((downloaded, failed))
    }

    /// 串行下载附件（小批量优化）
    fn download_attachments_sequential(
        &self,
        attachments: &[JiraAttachment],
        download_dir: &Path,
        url_resolver: Option<&UrlResolver>,
        callback: Option<&ProgressCallback>,
    ) -> Result<DownloadOperationResult> {
        let mut downloaded = Vec::new();
        let mut failed = Vec::new();

        for attachment in attachments {
            let file_path = download_dir.join(&attachment.filename);

            // 获取所有可能的 URL
            let urls = if let Some(resolver) = url_resolver {
                resolver.get_download_urls(attachment)
            } else {
                vec![attachment.content_url.clone()]
            };

            // 尝试下载
            match Self::try_download_attachment(attachment, &file_path, &urls) {
                Ok(path) => {
                    downloaded.push(path);
                    self.call_callback(callback, &format!("Downloaded: {}", attachment.filename));
                }
                Err(error_msg) => {
                    failed.push((attachment.filename.clone(), error_msg.clone()));
                    self.call_callback(
                        callback,
                        &format!(
                            "Failed to download: {} - {}",
                            attachment.filename, error_msg
                        ),
                    );
                }
            }
        }

        self.report_failed_downloads(&failed, callback);
        Ok((downloaded, failed))
    }

    /// 调用进度回调（如果存在）
    fn call_callback(&self, callback: Option<&ProgressCallback>, message: &str) {
        if let Some(cb) = callback {
            cb(message);
        }
    }

    /// 从任务结果中收集下载结果
    fn collect_download_results(
        &self,
        results: Vec<(String, TaskResult<PathBuf, String>)>,
    ) -> (Vec<PathBuf>, Vec<(String, String)>) {
        let mut downloaded = Vec::new();
        let mut failed = Vec::new();

        for (filename, result) in results {
            match result {
                TaskResult::Success(path) => downloaded.push(path),
                TaskResult::Failure(error) => failed.push((filename, error)),
            }
        }

        (downloaded, failed)
    }

    /// 报告失败的下载（提取公共逻辑）
    fn report_failed_downloads(
        &self,
        failed: &[(String, String)],
        callback: Option<&ProgressCallback>,
    ) {
        if !failed.is_empty() {
            self.call_callback(callback, "");
            self.call_callback(
                callback,
                &format!(
                    "  Warning: {} attachment(s) failed to download:",
                    failed.len()
                ),
            );
            for (filename, error) in failed {
                self.call_callback(callback, &format!("  - {}: {}", filename, error));
            }
        }
    }

    /// 处理下载的日志（合并分片、解压）
    fn process_downloaded_logs(
        &self,
        base_dir: &Path,
        download_dir: &Path,
        output_folder: &str,
        download_all_attachments: bool,
    ) -> Result<()> {
        let log_zip = download_dir.join(LOG_ZIP_FILENAME);
        let log_z01 = download_dir.join(format!("{}01", LOG_ZIP_SPLIT_PREFIX));

        if log_zip.exists() {
            // 检查是否有分片文件
            if log_z01.exists() {
                // 检测到分片文件，需要合并
                trace_debug!("Detected split files, merging...");
                self.zip_processor.merge_split_zips(download_dir)?;
            } else {
                // 单个 zip 文件，直接复制为 merged.zip
                let merged_zip = download_dir.join(MERGED_ZIP_FILENAME);
                std::fs::copy(&log_zip, &merged_zip).wrap_err_with(|| {
                    format!(
                        "Failed to copy {} to {}",
                        LOG_ZIP_FILENAME, MERGED_ZIP_FILENAME
                    )
                })?;
            }

            // 解压文件
            let extract_dir = if !output_folder.is_empty() {
                base_dir.join(output_folder)
            } else {
                base_dir.join(DEFAULT_OUTPUT_FOLDER)
            };

            let merged_zip = download_dir.join(MERGED_ZIP_FILENAME);
            if merged_zip.exists() {
                self.zip_processor.extract_zip(&merged_zip, &extract_dir)?;
            }
        } else if !download_all_attachments {
            // 检查是否有成功下载的日志文件（.txt, .log 等）
            let has_log_files = std::fs::read_dir(download_dir)?.filter_map(|e| e.ok()).any(|e| {
                if let Some(name) = e.file_name().to_str() {
                    LOG_EXTENSIONS.iter().any(|ext| name.ends_with(ext))
                } else {
                    false
                }
            });

            if !has_log_files {
                color_eyre::eyre::bail!(
                    "No log files found after download. All log attachments failed to download."
                );
            }
        }

        Ok(())
    }
}
