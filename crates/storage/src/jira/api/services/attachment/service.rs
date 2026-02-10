// //! 附件下载服务
// //!
// //! 提供附件下载的完整功能，包括解析、过滤、下载、ZIP 处理和目录管理。

// // Jira 附件下载常量定义
// const DEFAULT_OUTPUT_FOLDER: &str = "merged";
// const DOWNLOADS_FOLDER: &str = "downloads";
// const LOG_ZIP_FILENAME: &str = "log.zip";
// const MERGED_ZIP_FILENAME: &str = "merged.zip";
// const LOG_ZIP_SPLIT_PREFIX: &str = "log.z";
// const LOG_EXTENSIONS: &[&str] = &[".log", ".txt", ".zip"];

// use super::entity::UrlResolver;
// use crate::jira::api::services::attachment::Downloader;
// use crate::jira::api::services::traits::ProgressCallback;
// use crate::jira::api::services::AttachmentService;
// use crate::jira::api::services::IssueService;
// use crate::jira::types::JiraAttachment;
// use color_eyre::{eyre::eyre, eyre::WrapErr, Result};

// use domain::JiraConfigContext;
// use domain::{AttachmentDownloadResult, JiraError, JiraIssue};
// use std::path::{Path, PathBuf};
// use std::sync::mpsc;
// use std::sync::Arc;
// use std::thread;
// use toolkit::log_debug;
// use toolkit::util::ZipUtil;
// use toolkit::DirectoryWalker;
// use walkdir::WalkDir;

// /// 下载结果类型别名
// type DownloadResult = (Vec<PathBuf>, Vec<(String, String)>);

// /// 内部下载器（用于多线程环境）

// /// 附件下载服务
// pub struct AttachmentServiceImpl {
//     issue_service: Arc<dyn IssueService>,
//     config_context: Arc<dyn JiraConfigContext>,
//     zip_util: ZipUtil,
// }

// impl AttachmentService for AttachmentServiceImpl {
//     fn download_attachments(
//         &self,
//         issue_id: &str,
//         base_dir: &Path,
//         output_folder_name: &str,
//         download_all_attachments: bool,
//         callback: Option<ProgressCallback>,
//         max_concurrent: Option<usize>,
//         issue: Option<&domain::JiraIssue>,
//     ) -> Result<domain::AttachmentDownloadResult, domain::JiraError> {
//         self.download_attachments_impl(
//             issue_id,
//             base_dir,
//             output_folder_name,
//             download_all_attachments,
//             callback,
//             max_concurrent,
//             issue,
//         )
//     }

//     fn clean_attachments(&self, jira_id: Option<&str>) -> Result<(), domain::JiraError> {
//         self.clean_attachments_impl(jira_id)
//     }
// }

// impl AttachmentServiceImpl {
//     /// 创建新的 AttachmentService 实例
//     pub fn new(
//         issue_service: Arc<dyn IssueService>,
//         config_context: Arc<dyn JiraConfigContext>,
//     ) -> Self {
//         Self {
//             issue_service,
//             config_context,
//             zip_util: ZipUtil,
//         }
//     }

//     /// 下载附件实现
//     #[allow(clippy::too_many_arguments)]
//     fn download_attachments_impl(
//         &self,
//         issue_id: &str,
//         base_dir: &Path,
//         output_folder_name: &str,
//         download_all_attachments: bool,
//         callback: Option<ProgressCallback>,
//         max_concurrent: Option<usize>,
//         issue: Option<&JiraIssue>,
//     ) -> Result<AttachmentDownloadResult, JiraError> {
//         self.call_callback(callback.as_ref(), "Preparing download directory...");

//         // 1. 准备下载目录
//         let (download_base_dir, download_dir) = self
//             .prepare_download_directory(base_dir, issue_id)
//             .map_err(|e| JiraError::ApiError(format!("Failed to prepare directory: {}", e)))?;
//         let base_dir_path = download_base_dir.clone();

//         let mut result = AttachmentDownloadResult {
//             base_dir: download_base_dir.clone(),
//             downloaded_files: Vec::new(),
//             failed_files: Vec::new(),
//         };

//         let download_result = (|| -> Result<(), JiraError> {
//             // 2. 获取附件和描述数据
//             let (attachments, description) = if let Some(issue) = issue {
//                 self.call_callback(callback.as_ref(), "Using provided issue data...");
//                 let storage_attachments: Vec<JiraAttachment> = issue
//                     .attachments
//                     .iter()
//                     .map(|a| JiraAttachment {
//                         filename: a.filename.clone(),
//                         content_url: a.url.clone(),
//                         mime_type: None,
//                         size: Some(a.size),
//                     })
//                     .collect();
//                 (storage_attachments, issue.description.clone())
//             } else {
//                 self.call_callback(callback.as_ref(), "Fetching issue data...");
//                 let (_, storage_attachments, description) =
//                     self.issue_service.fetch_issue_data(issue_id).map_err(|e| {
//                         JiraError::ApiError(format!("Failed to fetch issue data: {}", e))
//                     })?;
//                 (storage_attachments, description)
//             };

//             if attachments.is_empty() {
//                 return Err(JiraError::ApiError(format!(
//                     "No attachments found for {}",
//                     issue_id
//                 )));
//             }

//             log_debug!("Found {} attachment(s):", attachments.len());
//             for attachment in &attachments {
//                 log_debug!("  - {}", attachment.filename);
//             }

//             // 3. 过滤附件
//             let target_attachments = if download_all_attachments {
//                 attachments.clone()
//             } else {
//                 let log_attachments = self.filter_log_attachments(&attachments);
//                 if log_attachments.is_empty() {
//                     return Err(JiraError::ApiError(format!(
//                         "No log attachments found for {}",
//                         issue_id
//                     )));
//                 }
//                 log_attachments
//             };

//             // 4. 下载附件
//             let max_concurrent = max_concurrent.unwrap_or(5).clamp(1, 20);
//             self.call_callback(
//                 callback.as_ref(),
//                 if download_all_attachments {
//                     "Downloading all attachments..."
//                 } else {
//                     "Downloading log attachments..."
//                 },
//             );

//             let url_resolver = if !download_all_attachments {
//                 Some(self.create_url_resolver(&target_attachments, description.as_deref()))
//             } else {
//                 None
//             };

//             let (downloaded, failed) = self
//                 .download_attachments_batch(
//                     &target_attachments,
//                     &download_dir,
//                     url_resolver.as_ref(),
//                     callback.as_ref(),
//                     max_concurrent,
//                 )
//                 .map_err(|e| {
//                     JiraError::ApiError(format!("Failed to download attachments: {}", e))
//                 })?;

//             result.downloaded_files.extend(downloaded);
//             result.failed_files.extend(failed);

//             self.call_callback(callback.as_ref(), "Processing downloaded logs...");

//             // 5. 处理下载的日志
//             self.process_downloaded_logs(
//                 &download_base_dir,
//                 &download_dir,
//                 output_folder_name,
//                 download_all_attachments,
//             )
//             .map_err(|e| JiraError::ApiError(format!("Failed to process logs: {}", e)))?;

//             Ok(())
//         })();

//         if download_result.is_err() && base_dir_path.exists() {
//             if let Err(e) = self.cleanup_on_failure(&base_dir_path) {
//                 log_debug!("Failed to cleanup directory on error: {}", e);
//             }
//             return download_result.map(|_| result);
//         }

//         download_result.map(|_| result)
//     }

//     /// 处理下载的日志（合并分片、解压）
//     fn process_downloaded_logs(
//         &self,
//         base_dir: &Path,
//         download_dir: &Path,
//         output_folder: &str,
//         download_all_attachments: bool,
//     ) -> Result<()> {
//         let log_zip = download_dir.join(LOG_ZIP_FILENAME);
//         let log_z01 = download_dir.join(format!("{}01", LOG_ZIP_SPLIT_PREFIX));

//         if log_zip.exists() {
//             if log_z01.exists() {
//                 log_debug!("Detected split files, merging...");
//                 self.merge_split_zips(download_dir)?;
//             } else {
//                 std::fs::copy(&log_zip, download_dir.join(MERGED_ZIP_FILENAME)).wrap_err_with(
//                     || {
//                         format!(
//                             "Failed to copy {} to {}",
//                             LOG_ZIP_FILENAME, MERGED_ZIP_FILENAME
//                         )
//                     },
//                 )?;
//             }

//             let extract_dir = if !output_folder.is_empty() {
//                 base_dir.join(output_folder)
//             } else {
//                 base_dir.join(DEFAULT_OUTPUT_FOLDER)
//             };

//             let merged_zip = download_dir.join(MERGED_ZIP_FILENAME);
//             if merged_zip.exists() {
//                 self.zip_util.extract(&merged_zip, &extract_dir)?;
//             }
//         } else if !download_all_attachments {
//             let has_log_files = std::fs::read_dir(download_dir)?
//                 .filter_map(|e| e.ok())
//                 .any(|e| {
//                     e.file_name()
//                         .to_str()
//                         .map(|name| LOG_EXTENSIONS.iter().any(|ext| name.ends_with(ext)))
//                         .unwrap_or(false)
//                 });

//             if !has_log_files {
//                 color_eyre::eyre::bail!(
//                     "No log files found after download. All log attachments failed to download."
//                 );
//             }
//         }

//         Ok(())
//     }

//     /// 合并分片 zip 文件
//     fn merge_split_zips(&self, download_dir: &Path) -> Result<PathBuf> {
//         let log_zip = download_dir.join(LOG_ZIP_FILENAME);
//         if !log_zip.exists() {
//             color_eyre::eyre::bail!("{} not found in {:?}", LOG_ZIP_FILENAME, download_dir);
//         }

//         let mut split_files: Vec<PathBuf> = WalkDir::new(download_dir)
//             .max_depth(1)
//             .into_iter()
//             .filter_map(|e| e.ok())
//             .filter(|e| {
//                 e.file_name()
//                     .to_str()
//                     .map(|name| {
//                         name.starts_with(LOG_ZIP_SPLIT_PREFIX)
//                             && name.len() == 8
//                             && name[6..].parse::<u8>().is_ok()
//                     })
//                     .unwrap_or(false)
//             })
//             .map(|e| e.path().to_path_buf())
//             .collect();

//         split_files.sort();

//         let merged_zip = download_dir.join(MERGED_ZIP_FILENAME);
//         if split_files.is_empty() {
//             std::fs::copy(&log_zip, &merged_zip).wrap_err_with(|| {
//                 format!(
//                     "Failed to copy {} to {}",
//                     LOG_ZIP_FILENAME, MERGED_ZIP_FILENAME
//                 )
//             })?;
//             return Ok(merged_zip);
//         }

//         self.zip_util
//             .merge_files(&log_zip, &split_files, &merged_zip)
//     }

//     // ==================== 辅助方法 ====================

//     /// 准备下载目录
//     fn prepare_download_directory(
//         &self,
//         base_dir: &Path,
//         jira_id: &str,
//     ) -> Result<(PathBuf, PathBuf)> {
//         let download_base_dir = base_dir.join("jira").join(jira_id);
//         let download_dir = download_base_dir.join(DOWNLOADS_FOLDER);

//         if download_base_dir.exists() {
//             std::fs::remove_dir_all(&download_base_dir)
//                 .wrap_err("Failed to remove existing directory")?;
//         }

//         DirectoryWalker::new(&download_dir).ensure_exists()?;
//         Ok((download_base_dir, download_dir))
//     }

//     /// 清理目录
//     fn cleanup_on_failure(&self, dir: &Path) -> Result<()> {
//         if dir.exists() {
//             std::fs::remove_dir_all(dir).wrap_err("Failed to cleanup directory")?;
//         }
//         Ok(())
//     }

//     /// 过滤日志附件
//     fn filter_log_attachments(&self, attachments: &[JiraAttachment]) -> Vec<JiraAttachment> {
//         let log_zip_pattern = regex::Regex::new(r"^log\.(zip|z\d+)$").unwrap();
//         let log_attachments: Vec<_> = attachments
//             .iter()
//             .filter(|a| {
//                 log_zip_pattern.is_match(&a.filename)
//                     || a.filename.ends_with(".log")
//                     || a.filename.ends_with(".txt")
//             })
//             .cloned()
//             .collect();

//         if !log_attachments.is_empty() {
//             log_debug!("Filtered {} log attachment(s):", log_attachments.len());
//             for attachment in &log_attachments {
//                 log_debug!("  - {}", attachment.filename);
//             }
//         }

//         log_attachments
//     }

//     /// 创建 URL 解析器
//     fn create_url_resolver(
//         &self,
//         attachments: &[JiraAttachment],
//         description: Option<&str>,
//     ) -> UrlResolver {
//         use std::collections::HashMap;
//         let mut api_map = HashMap::new();
//         for attachment in attachments {
//             api_map.insert(attachment.filename.clone(), attachment.content_url.clone());
//         }

//         let mut original_urls = HashMap::new();
//         if let Some(desc) = description {
//             let link_pattern = regex::Regex::new(r#"#\s*\[([^|]+)\|([^\]]+)\]"#).unwrap();
//             for cap in link_pattern.captures_iter(desc) {
//                 if let (Some(filename_match), Some(url_match)) = (cap.get(1), cap.get(2)) {
//                     let filename = filename_match.as_str().trim().to_string();
//                     let url = url_match.as_str().trim().to_string();
//                     if url.contains("cloudfront.net") {
//                         original_urls.insert(filename, url);
//                     }
//                 }
//             }
//         }

//         UrlResolver {
//             original_urls,
//             api_attachments_map: api_map,
//         }
//     }

//     /// 批量下载附件实现
//     fn download_attachments_batch(
//         &self,
//         attachments: &[JiraAttachment],
//         download_dir: &Path,
//         url_resolver: Option<&UrlResolver>,
//         callback: Option<&ProgressCallback>,
//         max_concurrent: usize,
//     ) -> Result<DownloadResult> {
//         let max_concurrent = max_concurrent.max(1).min(attachments.len().max(1));

//         if attachments.len() <= 1 {
//             return self.download_sequential(attachments, download_dir, url_resolver, callback);
//         }

//         let download_dir = download_dir.to_path_buf();
//         let config_context = Arc::clone(&self.config_context);
//         let mut tasks = Vec::new();

//         for attachment in attachments {
//             let download_dir = download_dir.clone();
//             let filename = attachment.filename.clone();
//             let attachment = attachment.clone();
//             let config_context = Arc::clone(&config_context);

//             let urls = if let Some(resolver) = url_resolver {
//                 resolver.get_download_urls(self.config_context.as_ref(), &attachment)
//             } else {
//                 vec![attachment.content_url.clone()]
//             };

//             let task = Box::new(move || -> Result<PathBuf, String> {
//                 let downloader = Downloader::new(config_context);
//                 let file_path = download_dir.join(&attachment.filename);
//                 downloader
//                     .try_download_file(&attachment, &file_path, &urls)
//                     .map_err(|e| e.to_string())
//             }) as Box<dyn FnOnce() -> Result<PathBuf, String> + Send>;

//             tasks.push((filename, task));
//         }

//         if tasks.is_empty() {
//             return Ok((Vec::new(), Vec::new()));
//         }

//         if tasks.len() == 1 {
//             let (name, task) = tasks
//                 .into_iter()
//                 .next()
//                 .ok_or_else(|| eyre!("Expected one task"))?;
//             let result = match task() {
//                 Ok(v) => {
//                     self.call_callback(callback, &format!("Downloaded: {}", name));
//                     Ok(v)
//                 }
//                 Err(e) => {
//                     self.call_callback(callback, &format!("Failed: {} - {}", name, e));
//                     Err(e)
//                 }
//             };
//             return Ok(self.collect_results(vec![(name, result)]));
//         }

//         let (tx, rx) = mpsc::channel();
//         let mut handles = Vec::new();
//         let mut tasks_iter = tasks.into_iter();

//         loop {
//             let mut chunk = Vec::new();
//             for _ in 0..max_concurrent {
//                 if let Some(task) = tasks_iter.next() {
//                     chunk.push(task);
//                 } else {
//                     break;
//                 }
//             }

//             if chunk.is_empty() {
//                 break;
//             }

//             let tx = tx.clone();
//             handles.push(thread::spawn(move || {
//                 for (name, task) in chunk {
//                     tx.send((name, task())).ok();
//                 }
//             }));
//         }

//         drop(tx);

//         let mut results = Vec::new();
//         for (name, result) in rx {
//             if let Some(cb) = callback {
//                 match &result {
//                     Ok(_) => cb(&format!("Downloaded: {}", name)),
//                     Err(e) => cb(&format!("Failed: {} - {}", name, e)),
//                 }
//             }
//             results.push((name, result));
//         }

//         for handle in handles {
//             handle
//                 .join()
//                 .map_err(|e| eyre!("Thread join error: {:?}", e))?;
//         }

//         let (downloaded, failed) = self.collect_results(results);
//         self.report_failed(&failed, callback);
//         Ok((downloaded, failed))
//     }

//     /// 串行下载
//     fn download_sequential(
//         &self,
//         attachments: &[JiraAttachment],
//         download_dir: &Path,
//         url_resolver: Option<&UrlResolver>,
//         callback: Option<&ProgressCallback>,
//     ) -> Result<DownloadResult> {
//         let mut downloaded = Vec::new();
//         let mut failed = Vec::new();

//         for attachment in attachments {
//             let file_path = download_dir.join(&attachment.filename);
//             let urls = if let Some(resolver) = url_resolver {
//                 resolver.get_download_urls(self.config_context.as_ref(), attachment)
//             } else {
//                 vec![attachment.content_url.clone()]
//             };

//             match self.try_download_file(attachment, &file_path, &urls) {
//                 Ok(path) => {
//                     downloaded.push(path);
//                     self.call_callback(callback, &format!("Downloaded: {}", attachment.filename));
//                 }
//                 Err(e) => {
//                     let error_msg = e.to_string();
//                     failed.push((attachment.filename.clone(), error_msg.clone()));
//                     self.call_callback(
//                         callback,
//                         &format!("Failed: {} - {}", attachment.filename, error_msg),
//                     );
//                 }
//             }
//         }

//         self.report_failed(&failed, callback);
//         Ok((downloaded, failed))
//     }

//     /// 尝试下载文件
//     fn try_download_file(
//         &self,
//         attachment: &JiraAttachment,
//         file_path: &Path,
//         urls: &[String],
//     ) -> Result<PathBuf> {
//         for url in urls {
//             match self.download_file(url, file_path) {
//                 Ok(()) => return Ok(file_path.to_path_buf()),
//                 Err(e) => log_debug!(
//                     "Failed to download {} from {}: {}",
//                     attachment.filename,
//                     url,
//                     e
//                 ),
//             }
//         }
//         Err(eyre!(
//             "Failed to download {} from all URLs",
//             attachment.filename
//         ))
//     }

//     /// 下载文件
//     fn download_file(&self, url: &str, output_path: &Path) -> Result<()> {
//         let downloader = Downloader::new(Arc::clone(&self.config_context));
//         downloader.download_file(url, output_path)
//     }

//     /// 收集结果
//     fn collect_results(
//         &self,
//         results: Vec<(String, Result<PathBuf, String>)>,
//     ) -> (Vec<PathBuf>, Vec<(String, String)>) {
//         let mut downloaded = Vec::new();
//         let mut failed = Vec::new();
//         for (filename, result) in results {
//             match result {
//                 Ok(path) => downloaded.push(path),
//                 Err(error) => failed.push((filename, error)),
//             }
//         }
//         (downloaded, failed)
//     }

//     /// 报告失败的下载
//     fn report_failed(&self, failed: &[(String, String)], callback: Option<&ProgressCallback>) {
//         if !failed.is_empty() {
//             if let Some(cb) = callback {
//                 cb("");
//                 cb(&format!(
//                     "  Warning: {} attachment(s) failed to download:",
//                     failed.len()
//                 ));
//                 for (filename, error) in failed {
//                     cb(&format!("  - {}: {}", filename, error));
//                 }
//             }
//         }
//     }

//     /// 调用回调
//     fn call_callback(&self, callback: Option<&ProgressCallback>, message: &str) {
//         if let Some(cb) = callback {
//             cb(message);
//         }
//     }

//     /// 清理附件目录实现（无交互）
//     ///
//     /// 清理指定 JIRA ID 的附件目录，或清理所有附件目录。
//     ///
//     /// # 参数
//     ///
//     /// * `jira_id` - JIRA ID（如 "PROJ-123"）。如果为 `None`，清理所有附件目录
//     ///
//     /// # 返回
//     ///
//     /// 如果清理成功，返回 `Ok(())`；否则返回错误。
//     fn clean_attachments_impl(&self, jira_id: Option<&str>) -> Result<(), JiraError> {
//         use std::path::PathBuf;
//         use toolkit::paths::default_download_base_dir;

//         // 获取基础目录
//         let base_dir = PathBuf::from(default_download_base_dir());

//         let dir = if let Some(id) = jira_id {
//             base_dir.join("jira").join(id)
//         } else {
//             base_dir.join("jira")
//         };

//         if !dir.exists() {
//             // 目录不存在，直接返回成功
//             return Ok(());
//         }

//         std::fs::remove_dir_all(&dir).map_err(|e| {
//             JiraError::ApiError(format!("Failed to delete directory {:?}: {}", dir, e))
//         })?;

//         Ok(())
//     }
// }
