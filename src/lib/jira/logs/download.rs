//! 下载功能相关实现

use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::jira::helpers::{get_auth, get_base_url};
use crate::jira::ticket::JiraTicket;
use crate::{log_break, log_debug, log_info, log_success, Jira, JiraAttachment};

use super::constants::*;
use super::JiraLogs;

impl JiraLogs {
    /// 从 Jira ticket 下载日志附件
    ///
    /// 返回下载的基础目录路径。
    pub fn download_from_jira(
        &self,
        jira_id: &str,
        log_output_folder_name: Option<&str>,
        download_all_attachments: bool,
    ) -> Result<PathBuf> {
        let output_folder = log_output_folder_name
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.output_folder_name.clone());

        // 1. 准备下载目录
        let (base_dir, download_dir) = self.prepare_download_directory(jira_id)?;

        // 2. 获取并过滤附件
        let (attachments, log_attachments) = self.fetch_and_filter_attachments(jira_id)?;

        // 3. 下载附件
        if download_all_attachments {
            self.download_all_attachments(&attachments, &download_dir)?;
        } else {
            if log_attachments.is_empty() {
                anyhow::bail!("No log attachments found for {}", jira_id);
            }
            self.download_log_attachments_with_retry(jira_id, &log_attachments, &download_dir)?;
        }

        // 4. 处理下载的日志（合并分片、解压）
        self.process_downloaded_logs(
            &base_dir,
            &download_dir,
            &output_folder,
            download_all_attachments,
        )?;

        Ok(base_dir)
    }

    /// 准备下载目录
    fn prepare_download_directory(&self, jira_id: &str) -> Result<(PathBuf, PathBuf)> {
        let base_dir = self.base_dir.join(jira_id);
        let download_dir = base_dir.join(DOWNLOADS_FOLDER);

        // 如果目录已存在，删除它
        if base_dir.exists() {
            std::fs::remove_dir_all(&base_dir).context("Failed to remove existing directory")?;
        }

        std::fs::create_dir_all(&download_dir).context("Failed to create output directory")?;

        Ok((base_dir, download_dir))
    }

    /// 获取并过滤附件
    fn fetch_and_filter_attachments(
        &self,
        jira_id: &str,
    ) -> Result<(Vec<JiraAttachment>, Vec<JiraAttachment>)> {
        // 获取附件列表
        let attachments: Vec<JiraAttachment> =
            Jira::get_attachments(jira_id).context("Failed to get attachments from Jira")?;

        if attachments.is_empty() {
            anyhow::bail!("No attachments found for {}", jira_id);
        }

        // 调试：显示所有附件
        log_debug!("Found {} attachment(s):", attachments.len());
        for attachment in &attachments {
            log_debug!("  - {}", attachment.filename);
        }

        // 过滤日志附件
        // 匹配规则（与 shell 脚本的 awk 模式一致）：
        // 1. log.zip 或 log.z[0-9]+ 格式的文件（如 log.zip, log.z01, log.z02 等）
        // 2. 以 .log 结尾的文件（如 any_file.log, error.log 等）
        // 3. 以 .txt 结尾的文件（如 metric0.txt, log0.txt, network3.txt 等）
        let log_zip_pattern = Regex::new(r"^log\.(zip|z\d+)$")?;
        let log_attachments: Vec<_> = attachments
            .iter()
            .filter(|a| {
                // 匹配 log.zip 或 log.z[0-9]+ 格式（与 shell 脚本一致）
                let matches_log_zip = log_zip_pattern.is_match(&a.filename);
                // 匹配所有以 .log 结尾的文件
                let matches_log_ext = a.filename.ends_with(".log");
                // 匹配所有以 .txt 结尾的文件
                let matches_txt_ext = a.filename.ends_with(".txt");

                matches_log_zip || matches_log_ext || matches_txt_ext
            })
            .cloned()
            .collect();

        // 调试：显示过滤后的日志附件
        if !log_attachments.is_empty() {
            log_debug!("Filtered {} log attachment(s):", log_attachments.len());
            for attachment in &log_attachments {
                log_debug!("  - {}", attachment.filename);
            }
        }

        Ok((attachments, log_attachments))
    }

    /// 下载所有附件
    fn download_all_attachments(
        &self,
        attachments: &[JiraAttachment],
        download_dir: &Path,
    ) -> Result<()> {
        for attachment in attachments {
            let file_path = download_dir.join(&attachment.filename);
            self.download_file(&attachment.content_url, &file_path)?;
        }
        Ok(())
    }

    /// 下载日志附件（带重试逻辑）
    fn download_log_attachments_with_retry(
        &self,
        jira_id: &str,
        log_attachments: &[JiraAttachment],
        download_dir: &Path,
    ) -> Result<()> {
        // 提取 URL 映射
        let (original_urls, api_attachments_map) = self.extract_url_mappings(jira_id)?;

        let mut failed_attachments = Vec::new();
        for attachment in log_attachments {
            let file_path = download_dir.join(&attachment.filename);
            if let Err(error) = self.download_attachment_with_retry(
                attachment,
                &file_path,
                &original_urls,
                &api_attachments_map,
            ) {
                failed_attachments.push((attachment.filename.clone(), error));
            }
        }

        // 如果有失败的附件，显示警告但不中断整个流程
        if !failed_attachments.is_empty() {
            log_break!();
            log_info!(
                "  Warning: {} attachment(s) failed to download:",
                failed_attachments.len()
            );
            for (filename, error) in &failed_attachments {
                log_info!("  - {}: {}", filename, error);
            }
        }

        Ok(())
    }

    /// 提取 URL 映射（从 ticket 描述和附件列表）
    fn extract_url_mappings(
        &self,
        jira_id: &str,
    ) -> Result<(HashMap<String, String>, HashMap<String, String>)> {
        // 预先编译正则表达式，避免在循环中重复编译
        let link_pattern = Regex::new(r#"#\s*\[([^|]+)\|([^\]]+)\]"#).unwrap();

        // 预先获取 ticket 信息（只调用一次 API）
        let issue = Jira::get_ticket_info(jira_id).ok();

        // 从描述中提取原始 URL 映射
        let mut original_urls: HashMap<String, String> = HashMap::new();
        if let Some(ref issue) = issue {
            if let Some(description) = &issue.fields.description {
                for cap in link_pattern.captures_iter(description) {
                    if let (Some(filename_match), Some(url_match)) = (cap.get(1), cap.get(2)) {
                        let filename = filename_match.as_str().trim().to_string();
                        let url = url_match.as_str().trim().to_string();
                        if url.contains("cloudfront.net") {
                            original_urls.insert(filename, url);
                        }
                    }
                }
            }
        }

        // 从附件列表中提取 URL 映射
        let mut api_attachments_map: HashMap<String, String> = HashMap::new();
        if let Some(ref issue) = issue {
            if let Some(api_attachments) = &issue.fields.attachment {
                for api_att in api_attachments {
                    api_attachments_map
                        .insert(api_att.filename.clone(), api_att.content_url.clone());
                }
            }
        }

        Ok((original_urls, api_attachments_map))
    }

    /// 下载单个附件（带重试逻辑）
    fn download_attachment_with_retry(
        &self,
        attachment: &JiraAttachment,
        file_path: &Path,
        original_urls: &HashMap<String, String>,
        api_attachments_map: &HashMap<String, String>,
    ) -> Result<()> {
        // 首先尝试使用当前的 URL
        if self
            .download_file(&attachment.content_url, file_path)
            .is_ok()
        {
            log_success!("Downloaded: {}", attachment.filename);
            return Ok(());
        }

        let original_error = format!(
            "Failed to download {} from primary URL",
            attachment.filename
        );
        log_debug!(
            "Warning: Failed to download {}: {}",
            attachment.filename,
            original_error
        );

        // 如果当前 URL 是 CloudFront URL，尝试多种方式
        if attachment.content_url.contains("cloudfront.net") {
            // 方式 1: 从 API 附件列表中查找
            if let Some(api_url) = api_attachments_map.get(&attachment.filename) {
                log_debug!(
                    "Trying Jira API URL for {}: {}",
                    attachment.filename,
                    api_url
                );
                if self.download_file(api_url, file_path).is_ok() {
                    log_success!("Downloaded: {} (using Jira API URL)", attachment.filename);
                    return Ok(());
                }
            }

            // 方式 2: 从 CloudFront URL 中提取附件 ID 并构建 Jira API URL
            if let Some(attachment_id) =
                JiraTicket::extract_attachment_id_from_url(&attachment.content_url)
            {
                if let Ok(base_url) = get_base_url() {
                    let jira_api_url = format!("{}/attachment/content/{}", base_url, attachment_id);
                    log_debug!(
                        "Trying Jira API URL from attachment ID {} for {}: {}",
                        attachment_id,
                        attachment.filename,
                        jira_api_url
                    );
                    if self.download_file(&jira_api_url, file_path).is_ok() {
                        log_success!(
                            "Downloaded: {} (using Jira API URL from attachment ID)",
                            attachment.filename
                        );
                        return Ok(());
                    }
                }
            }
        }

        // 如果还是失败，尝试使用原始 CloudFront URL（如果不同）
        if let Some(original_url) = original_urls.get(&attachment.filename) {
            if original_url != &attachment.content_url {
                log_debug!(
                    "Retrying with original CloudFront URL for {}",
                    attachment.filename
                );
                if self.download_file(original_url, file_path).is_ok() {
                    log_success!(
                        "Downloaded: {} (using original CloudFront URL)",
                        attachment.filename
                    );
                    return Ok(());
                }
            }
        }

        // 所有重试都失败
        anyhow::bail!("{}", original_error)
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
                log_debug!("Detected split files, merging...");
                self.merge_split_zips(download_dir)?;
            } else {
                // 单个 zip 文件，直接复制为 merged.zip
                let merged_zip = download_dir.join(MERGED_ZIP_FILENAME);
                std::fs::copy(&log_zip, &merged_zip).with_context(|| {
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
                self.extract_zip(&merged_zip, &extract_dir)?;
            }
        } else if !download_all_attachments {
            // 检查是否有成功下载的日志文件（.txt, .log 等）
            let has_log_files = std::fs::read_dir(download_dir)?
                .filter_map(|e| e.ok())
                .any(|e| {
                    if let Some(name) = e.file_name().to_str() {
                        LOG_EXTENSIONS.iter().any(|ext| name.ends_with(ext))
                    } else {
                        false
                    }
                });

            if !has_log_files {
                anyhow::bail!(
                    "No log files found after download. All log attachments failed to download."
                );
            }
        }

        Ok(())
    }

    /// 格式化下载错误信息
    fn format_download_error(&self, status: reqwest::StatusCode, error_text: String) -> String {
        if !error_text.is_empty() {
            let preview = if error_text.len() > 200 {
                format!("{}...", &error_text[..200])
            } else {
                error_text
            };
            format!("Download failed with status: {} - {}", status, preview)
        } else {
            format!("Download failed with status: {}", status)
        }
    }

    /// 下载单个文件
    fn download_file(&self, url: &str, output_path: &Path) -> Result<()> {
        // 获取 Jira 认证信息
        let (email, api_token) = get_auth()?;

        // 判断是否是 CloudFront 签名 URL（包含 Expires 和 Signature 参数）
        // CloudFront 签名 URL 通常不需要 Basic Auth，或者 Basic Auth 会干扰签名验证
        let is_cloudfront_signed_url = url.contains("cloudfront.net")
            && url.contains("Expires=")
            && url.contains("Signature=");

        // 获取 Jira base URL 用于 Referer 头
        let jira_base_url = get_base_url().ok();

        let mut response = if is_cloudfront_signed_url {
            // CloudFront 签名 URL，先尝试不使用 Basic Auth，但添加 Referer 头
            log_debug!("Using CloudFront signed URL without Basic Auth");
            let mut request = self.http_client.get(url);

            // 添加 Referer 头，可能有助于 CloudFront 验证
            if let Some(ref base_url) = jira_base_url {
                request = request.header("Referer", base_url);
            }

            request
                .send()
                .with_context(|| format!("Failed to download: {}", url))?
        } else {
            // Jira API URL，使用 Basic Auth
            self.http_client
                .get(url)
                .basic_auth(&email, Some(&api_token))
                .send()
                .with_context(|| format!("Failed to download: {}", url))?
        };

        if !response.status().is_success() {
            // 如果 CloudFront URL 失败，尝试使用 Basic Auth
            if is_cloudfront_signed_url {
                let status = response.status();
                log_debug!(
                    "CloudFront URL failed (status: {}), retrying with Basic Auth",
                    status
                );

                // 尝试读取响应体以获取更多错误信息
                let error_text = response.text().unwrap_or_default();
                if !error_text.is_empty() {
                    let preview = if error_text.len() > 200 {
                        format!("{}...", &error_text[..200])
                    } else {
                        error_text.clone()
                    };
                    log_debug!("Error response: {}", preview);
                }

                let mut request = self.http_client.get(url);
                // 添加 Referer 头
                if let Some(ref base_url) = jira_base_url {
                    request = request.header("Referer", base_url);
                }

                response = request
                    .basic_auth(&email, Some(&api_token))
                    .send()
                    .with_context(|| format!("Failed to download with Basic Auth: {}", url))?;

                if !response.status().is_success() {
                    let status = response.status();
                    let error_text = response.text().unwrap_or_default();
                    let error_msg = self.format_download_error(status, error_text);
                    anyhow::bail!("{}", error_msg);
                }
            } else {
                let status = response.status();
                let error_text = response.text().unwrap_or_default();
                let error_msg = self.format_download_error(status, error_text);
                anyhow::bail!("{}", error_msg);
            }
        }

        let mut file = File::create(output_path)
            .with_context(|| format!("Failed to create file: {:?}", output_path))?;

        std::io::copy(&mut response, &mut file)
            .with_context(|| format!("Failed to write file: {:?}", output_path))?;

        Ok(())
    }
}
