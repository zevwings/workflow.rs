//! 附件下载器实现
//!
//! 提供附件下载的核心功能，包括并发下载、URL 重试等。

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::{mpsc, Arc},
    thread,
};

use domain::{JiraAttachment, JiraConfigContext, JiraError};
use http::Authorization;
use reqwest::header::HeaderMap;

/// 进度回调函数类型
pub type ProgressCallback = Box<dyn Fn(&str) + Send + Sync>;

/// 下载结果类型别名：(成功文件列表, 失败文件列表)
pub type DownloadOperationResult = (Vec<PathBuf>, Vec<(String, String)>);

/// 任务结果枚举
enum TaskResult<T, E> {
    Success(T),
    Failure(E),
}

/// HTTP 下载器
///
/// 负责实际的文件下载操作。
pub struct FileDownloader {
    config_context: Arc<dyn JiraConfigContext>,
}

impl FileDownloader {
    /// 创建新的 HTTP 下载器
    pub fn new(config_context: Arc<dyn JiraConfigContext>) -> Self {
        Self { config_context }
    }

    /// 尝试从多个 URL 下载文件
    ///
    /// # 参数
    ///
    /// * `attachment` - 附件信息
    /// * `file_path` - 输出文件路径
    /// * `urls` - URL 列表（按优先级排序）
    pub fn try_download_file(
        &self,
        attachment: &JiraAttachment,
        file_path: &Path,
        urls: &[String],
    ) -> Result<PathBuf, String> {
        for url in urls {
            match self.download_file(url, file_path) {
                Ok(()) => return Ok(file_path.to_path_buf()),
                Err(e) => {
                    // 记录失败但继续尝试下一个 URL
                    eprintln!(
                        "Failed to download {} from {}: {}",
                        attachment.filename, url, e
                    );
                }
            }
        }
        Err(format!(
            "Failed to download {} from all {} URL(s)",
            attachment.filename,
            urls.len()
        ))
    }

    /// 下载单个文件
    ///
    /// # 参数
    ///
    /// * `url` - 下载 URL
    /// * `output_path` - 输出文件路径
    fn download_file(&self, url: &str, output_path: &Path) -> Result<(), JiraError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| JiraError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        let mut headers = HeaderMap::new();

        // 添加 Referer header
        if let Ok(base_url) = self.config_context.get_base_url() {
            if let Ok(referer) = base_url.parse() {
                headers.insert("Referer", referer);
            }
        }

        // 检查是否为 CloudFront URL
        let is_cloudfront = url.contains("cloudfront.net")
            && url.contains("Expires=")
            && url.contains("Signature=");

        // 非 CloudFront URL 使用 Basic Auth
        if !is_cloudfront {
            let (email, api_token) = self.config_context.get_auth()?;
            let auth = Authorization::basic(email, api_token);
            auth.apply_to_headers(&mut headers)
                .map_err(|e| JiraError::ApiError(format!("Failed to apply auth headers: {}", e)))?;
        }

        let request = client.get(url).headers(headers.clone());

        let mut response = request.send().map_err(|e| {
            JiraError::NetworkError(format!("Failed to send request to {}: {}", url, e))
        })?;

        // 如果 CloudFront 失败，尝试使用 Basic Auth 重试
        if !response.status().is_success() && is_cloudfront {
            let (email, api_token) = self.config_context.get_auth()?;
            let mut retry_headers = HeaderMap::new();

            // 重新添加 Referer
            if let Ok(base_url) = self.config_context.get_base_url() {
                if let Ok(referer) = base_url.parse() {
                    retry_headers.insert("Referer", referer);
                }
            }

            let auth = Authorization::basic(email, api_token);
            auth.apply_to_headers(&mut retry_headers).map_err(|e| {
                JiraError::ApiError(format!("Failed to apply retry auth headers: {}", e))
            })?;

            let retry_request = client.get(url).headers(retry_headers);

            response = retry_request.send().map_err(|e| {
                JiraError::NetworkError(format!("Failed to retry request with Basic Auth: {}", e))
            })?;
        }

        // 检查响应状态
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            let preview = if error_text.len() > 200 {
                format!("{}...", &error_text[..200])
            } else {
                error_text
            };
            return Err(JiraError::ApiError(format!(
                "Download failed with status {}: {}",
                status, preview
            )));
        }

        // 写入文件
        let mut file = File::create(output_path).map_err(|e| {
            JiraError::IoError(format!("Failed to create file {:?}: {}", output_path, e))
        })?;

        let bytes = response
            .bytes()
            .map_err(|e| JiraError::NetworkError(format!("Failed to read response body: {}", e)))?;

        file.write_all(&bytes).map_err(|e| {
            JiraError::IoError(format!("Failed to write file {:?}: {}", output_path, e))
        })?;

        Ok(())
    }
}

/// 并发下载管理器
///
/// 负责协调并发下载任务。
pub struct ConcurrentDownloader {
    downloader: Arc<FileDownloader>,
}

impl ConcurrentDownloader {
    /// 创建新的并发下载管理器
    pub fn new(context: Arc<dyn JiraConfigContext>) -> Self {
        Self {
            downloader: Arc::new(FileDownloader::new(context)),
        }
    }

    /// 并发下载多个附件
    ///
    /// # 参数
    ///
    /// * `tasks` - 下载任务列表（文件名, 附件, 文件路径, URL列表）
    /// * `max_concurrent` - 最大并发数
    /// * `callback` - 进度回调函数（可选）
    pub fn download_concurrent(
        &self,
        tasks: Vec<(String, JiraAttachment, PathBuf, Vec<String>)>,
        max_concurrent: usize,
        callback: Option<&ProgressCallback>,
    ) -> DownloadOperationResult {
        // 优化：小批量使用串行下载
        if tasks.len() <= 1 {
            return self.download_sequential(tasks, callback);
        }

        let max_concurrent = max_concurrent.max(1).min(tasks.len().max(1));

        // 创建结果通道
        let (tx, rx) = mpsc::channel();

        // 分批处理任务
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
            let downloader = self.downloader.clone();

            let handle = thread::spawn(move || {
                for (name, attachment, file_path, urls) in chunk {
                    let result = downloader.try_download_file(&attachment, &file_path, &urls);
                    let task_result = match result {
                        Ok(path) => TaskResult::Success(path),
                        Err(err) => TaskResult::Failure(err),
                    };
                    tx.send((name, task_result)).ok();
                }
            });

            handles.push(handle);
        }

        // 关闭发送端
        drop(tx);

        // 收集结果，并实时调用回调
        let mut downloaded = Vec::new();
        let mut failed = Vec::new();

        for (name, result) in rx {
            match result {
                TaskResult::Success(path) => {
                    downloaded.push(path);
                    if let Some(cb) = callback {
                        cb(&format!("Downloaded: {}", name));
                    }
                }
                TaskResult::Failure(error) => {
                    failed.push((name.clone(), error.clone()));
                    if let Some(cb) = callback {
                        cb(&format!("Failed to download: {} - {}", name, error));
                    }
                }
            }
        }

        // 等待所有线程完成
        for handle in handles {
            let _ = handle.join();
        }

        (downloaded, failed)
    }

    /// 串行下载（小批量优化）
    fn download_sequential(
        &self,
        tasks: Vec<(String, JiraAttachment, PathBuf, Vec<String>)>,
        callback: Option<&ProgressCallback>,
    ) -> DownloadOperationResult {
        let mut downloaded = Vec::new();
        let mut failed = Vec::new();

        for (name, attachment, file_path, urls) in tasks {
            match self.downloader.try_download_file(&attachment, &file_path, &urls) {
                Ok(path) => {
                    downloaded.push(path);
                    if let Some(cb) = callback {
                        cb(&format!("Downloaded: {}", name));
                    }
                }
                Err(error) => {
                    failed.push((name.clone(), error.clone()));
                    if let Some(cb) = callback {
                        cb(&format!("Failed to download: {} - {}", name, error));
                    }
                }
            }
        }

        (downloaded, failed)
    }
}
