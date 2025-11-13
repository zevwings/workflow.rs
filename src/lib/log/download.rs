//! 下载模块
//! 从 Jira 下载日志附件，处理 zip 文件

use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::jira::helpers::get_auth;
use crate::{log_break, log_debug, log_info, log_success, Jira, Settings};

use super::utils::expand_path;
use super::zip::{extract_zip, merge_split_zips};

/// 格式化下载错误信息
fn format_download_error(status: reqwest::StatusCode, error_text: String) -> String {
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
fn download_file(url: &str, output_path: &Path) -> Result<()> {
    // 获取 Jira 认证信息
    let (email, api_token) = get_auth()?;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .context("Failed to create HTTP client")?;

    // 判断是否是 CloudFront 签名 URL（包含 Expires 和 Signature 参数）
    // CloudFront 签名 URL 通常不需要 Basic Auth，或者 Basic Auth 会干扰签名验证
    let is_cloudfront_signed_url =
        url.contains("cloudfront.net") && url.contains("Expires=") && url.contains("Signature=");

    // 获取 Jira base URL 用于 Referer 头
    let jira_base_url = crate::jira::helpers::get_base_url().ok();

    let mut response = if is_cloudfront_signed_url {
        // CloudFront 签名 URL，先尝试不使用 Basic Auth，但添加 Referer 头
        log_debug!("Using CloudFront signed URL without Basic Auth");
        let mut request = client.get(url);

        // 添加 Referer 头，可能有助于 CloudFront 验证
        if let Some(ref base_url) = jira_base_url {
            request = request.header("Referer", base_url);
        }

        request
            .send()
            .with_context(|| format!("Failed to download: {}", url))?
    } else {
        // Jira API URL，使用 Basic Auth
        client
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

            let mut request = client.get(url);
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
                let error_msg = format_download_error(status, error_text);
                anyhow::bail!("{}", error_msg);
            }
        } else {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            let error_msg = format_download_error(status, error_text);
            anyhow::bail!("{}", error_msg);
        }
    }

    let mut file = File::create(output_path)
        .with_context(|| format!("Failed to create file: {:?}", output_path))?;

    std::io::copy(&mut response, &mut file)
        .with_context(|| format!("Failed to write file: {:?}", output_path))?;

    Ok(())
}

/// 从 Jira ticket 下载日志附件
/// 返回下载的基础目录路径
pub fn download_from_jira(
    jira_id: &str,
    log_output_folder_name: Option<&str>,
    download_all_attachments: bool,
) -> Result<PathBuf> {
    // 1. 确定输出目录
    let settings = Settings::get();
    let base_dir_str = settings.log.download_base_dir.clone().unwrap_or_default();

    // 展开 ~ 路径
    let base_dir = expand_path(&base_dir_str)?;

    // 每个 JIRA ticket 使用独立的子目录
    let base_dir = base_dir.join(jira_id);

    // 创建 downloads 子目录（与 shell 脚本一致）
    let download_dir = base_dir.join("downloads");

    // 如果目录已存在，删除它
    if base_dir.exists() {
        std::fs::remove_dir_all(&base_dir).context("Failed to remove existing directory")?;
    }

    std::fs::create_dir_all(&download_dir).context("Failed to create output directory")?;

    // 2. 获取附件列表
    let attachments: Vec<crate::JiraAttachment> =
        Jira::get_attachments(jira_id).context("Failed to get attachments from Jira")?;

    if attachments.is_empty() {
        anyhow::bail!("No attachments found for {}", jira_id);
    }

    // 调试：显示所有附件
    log_debug!("Found {} attachment(s):", attachments.len());
    for attachment in &attachments {
        log_debug!("  - {}", attachment.filename);
    }

    // 3. 过滤日志附件
    // 匹配规则（与 shell 脚本的 awk 模式一致）：
    // 1. log.zip 或 log.z[0-9]+ 格式的文件（如 log.zip, log.z01, log.z02 等）
    // 2. 以 .log 结尾的文件（如 any_file.log, error.log 等）
    // 3. 以 .txt 结尾的文件（如 metric0.txt, log0.txt, network3.txt 等）
    // Shell 脚本使用: /^[[:space:]]*[0-9]+\. log\.(zip|z[0-9]+)[[:space:]]*$/
    // 我们简化匹配：log\.(zip|z[0-9]+) 或 log\d*\.(zip|z[0-9]+)
    let log_zip_pattern = Regex::new(r"^log\.(zip|z\d+)$")?;
    let log_attachments: Vec<_> = attachments
        .iter()
        .filter(|a| {
            // 匹配 log.zip 或 log.z[0-9]+ 格式（与 shell 脚本一致）
            // 例如：log.zip, log.z01, log.z02 等
            let matches_log_zip = log_zip_pattern.is_match(&a.filename);
            // 匹配所有以 .log 结尾的文件
            // 例如：any_file.log, error.log, debug.log 等
            let matches_log_ext = a.filename.ends_with(".log");
            // 匹配所有以 .txt 结尾的文件
            // 例如：metric0.txt, log0.txt, network3.txt, any_file.txt 等
            let matches_txt_ext = a.filename.ends_with(".txt");

            matches_log_zip || matches_log_ext || matches_txt_ext
        })
        .collect();

    // 调试：显示过滤后的日志附件
    if !log_attachments.is_empty() {
        log_debug!("Filtered {} log attachment(s):", log_attachments.len());
        for attachment in &log_attachments {
            log_debug!("  - {}", attachment.filename);
        }
    }

    // 4. 下载附件
    if download_all_attachments {
        // 下载所有附件到 downloads 目录
        for attachment in &attachments {
            let file_path = download_dir.join(&attachment.filename);
            download_file(&attachment.content_url, &file_path)?;
        }
    } else {
        // 只下载日志附件
        if log_attachments.is_empty() {
            anyhow::bail!("No log attachments found for {}", jira_id);
        }

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

        let mut failed_attachments = Vec::new();
        for attachment in &log_attachments {
            let file_path = download_dir.join(&attachment.filename);

            // 首先尝试使用当前的 URL
            let (mut download_success, original_error) = match download_file(
                &attachment.content_url,
                &file_path,
            ) {
                Ok(()) => {
                    log_success!("Downloaded: {}", attachment.filename);
                    (true, None)
                }
                Err(e) => {
                    log_debug!("Warning: Failed to download {}: {}", attachment.filename, e);
                    let error_msg = format!("{}", e);

                    // 如果当前 URL 是 CloudFront URL，尝试多种方式：
                    // 1. 从 Jira API 附件列表中查找匹配的文件名
                    // 2. 从 CloudFront URL 中提取附件 ID 并构建 Jira API URL
                    let success = if attachment.content_url.contains("cloudfront.net") {
                        let mut success = false;

                        // 方式 1: 从 API 附件列表中查找
                        if let Some(api_url) = api_attachments_map.get(&attachment.filename) {
                            log_debug!(
                                "Trying Jira API URL for {}: {}",
                                attachment.filename,
                                api_url
                            );
                            match download_file(api_url, &file_path) {
                                Ok(()) => {
                                    log_success!(
                                        "Downloaded: {} (using Jira API URL)",
                                        attachment.filename
                                    );
                                    success = true;
                                }
                                Err(e2) => {
                                    log_debug!("Also failed with Jira API URL: {}", e2);
                                }
                            }
                        }

                        // 方式 2: 从 CloudFront URL 中提取附件 ID 并构建 Jira API URL
                        if !success {
                            if let Some(attachment_id) =
                                crate::jira::ticket::extract_attachment_id_from_url(
                                    &attachment.content_url,
                                )
                            {
                                if let Ok(base_url) = crate::jira::helpers::get_base_url() {
                                    let jira_api_url = format!(
                                        "{}/attachment/content/{}",
                                        base_url, attachment_id
                                    );
                                    log_debug!(
                                        "Trying Jira API URL from attachment ID {} for {}: {}",
                                        attachment_id,
                                        attachment.filename,
                                        jira_api_url
                                    );
                                    match download_file(&jira_api_url, &file_path) {
                                        Ok(()) => {
                                            log_success!("Downloaded: {} (using Jira API URL from attachment ID)", attachment.filename);
                                            success = true;
                                        }
                                        Err(e2) => {
                                            log_debug!("Also failed with Jira API URL from attachment ID: {}", e2);
                                        }
                                    }
                                }
                            }
                        }

                        success
                    } else {
                        false
                    };

                    (success, Some(error_msg))
                }
            };

            // 如果还是失败，尝试使用原始 CloudFront URL（如果不同）
            if !download_success {
                if let Some(original_url) = original_urls.get(&attachment.filename) {
                    if original_url != &attachment.content_url {
                        log_debug!(
                            "Retrying with original CloudFront URL for {}",
                            attachment.filename
                        );
                        download_success = match download_file(original_url, &file_path) {
                            Ok(()) => {
                                log_success!(
                                    "Downloaded: {} (using original CloudFront URL)",
                                    attachment.filename
                                );
                                true
                            }
                            Err(e2) => {
                                log_debug!(
                                    "Warning: Also failed with original CloudFront URL: {}",
                                    e2
                                );
                                false
                            }
                        };
                    }
                }
            }

            if !download_success {
                // 使用保存的原始错误信息
                if let Some(error) = original_error {
                    failed_attachments
                        .push((attachment.filename.clone(), anyhow::anyhow!("{}", error)));
                }
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
    }

    // 5. 处理日志附件（合并分片、解压）
    // 检查是否有 log.zip 或分片文件（与 shell 脚本一致）
    let log_zip = download_dir.join("log.zip");
    let log_z01 = download_dir.join("log.z01");

    if log_zip.exists() {
        // 检查是否有分片文件（与 shell 脚本一致：检查 log.z01）
        if log_z01.exists() {
            // 检测到分片文件，需要合并（与 shell 脚本一致）
            log_debug!("检测到分片文件，需要合并...");
            merge_split_zips(&download_dir)?;
        } else {
            // 单个 zip 文件，直接复制为 merged.zip
            let merged_zip = download_dir.join("merged.zip");
            std::fs::copy(&log_zip, &merged_zip).context("Failed to copy log.zip to merged.zip")?;
        }

        // 解压文件
        let extract_dir = if let Some(folder_name) = log_output_folder_name {
            base_dir.join(folder_name)
        } else {
            base_dir.join("merged")
        };

        let merged_zip = download_dir.join("merged.zip");
        if merged_zip.exists() {
            extract_zip(&merged_zip, &extract_dir)?;
        }
    } else if !download_all_attachments {
        // 检查是否有成功下载的日志文件（.txt, .log 等）
        let has_log_files = std::fs::read_dir(&download_dir)?
            .filter_map(|e| e.ok())
            .any(|e| {
                if let Some(name) = e.file_name().to_str() {
                    name.ends_with(".txt") || name.ends_with(".log") || name.ends_with(".zip")
                } else {
                    false
                }
            });

        if !has_log_files {
            // 如果没有日志附件且不是下载所有附件，返回错误
            anyhow::bail!(
                "No log files found after download. All log attachments failed to download."
            );
        }
    }

    Ok(base_dir)
}
