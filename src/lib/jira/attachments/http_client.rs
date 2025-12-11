//! HTTP 客户端适配器（用于附件下载）

use crate::base::http::{Authorization, HttpClient, HttpMethod, RequestConfig};
use crate::jira::helpers::{get_auth, get_base_url};
use crate::trace_debug;
use anyhow::{Context, Result};
use reqwest::header::HeaderMap;
use std::fs::File;
use std::path::Path;

/// 附件下载器
///
/// 提供文件下载功能，利用现有的 `base::http::HttpClient` 进行流式下载。
pub struct AttachmentDownloader;

impl AttachmentDownloader {
    /// 检查是否为 CloudFront 签名 URL
    ///
    /// CloudFront 签名 URL 包含 Expires 和 Signature 参数。
    fn is_cloudfront_signed_url(url: &str) -> bool {
        url.contains("cloudfront.net") && url.contains("Expires=") && url.contains("Signature=")
    }

    /// 下载文件到指定路径
    ///
    /// 利用现有的 `base::http::HttpClient` 进行流式下载。
    /// 支持 CloudFront 签名 URL 的特殊处理（先尝试不使用 Basic Auth，失败后重试）。
    ///
    /// # 参数
    ///
    /// * `url` - 下载 URL
    /// * `output_path` - 输出文件路径
    ///
    /// # 返回
    ///
    /// 如果下载成功，返回 `Ok(())`；否则返回错误。
    pub fn download_file(url: &str, output_path: &Path) -> Result<()> {
        let client = HttpClient::global()?;

        // 构建请求头
        let mut headers = HeaderMap::new();
        if let Ok(base_url) = get_base_url() {
            if let Ok(referer_header) = base_url.parse() {
                headers.insert("Referer", referer_header);
            }
        }

        let is_cloudfront = Self::is_cloudfront_signed_url(url);

        // 构建第一次请求配置
        let mut config: RequestConfig<'_, serde_json::Value, serde_json::Value> =
            RequestConfig::new().timeout(std::time::Duration::from_secs(60));
        if !headers.is_empty() {
            config = config.headers(&headers);
        }

        // 对于非 CloudFront URL，添加 Basic Auth
        let auth = if !is_cloudfront {
            let (email, api_token) = get_auth()?;
            Some(Authorization::new(email, api_token))
        } else {
            None
        };
        if let Some(ref auth_ref) = auth {
            config = config.auth(auth_ref);
        }

        // 第一次尝试
        let mut response = client
            .stream(HttpMethod::Get, url, config)
            .with_context(|| format!("Failed to download: {}", url))?;

        // 如果失败且是 CloudFront URL，重试时使用 Basic Auth
        if !response.status().is_success() && is_cloudfront {
            let status = response.status();
            trace_debug!(
                "CloudFront URL failed (status: {}), retrying with Basic Auth",
                status
            );

            // 尝试读取响应体以获取更多错误信息
            let error_text = response.text().unwrap_or_default();
            if !error_text.is_empty() {
                let preview = if error_text.len() > 200 {
                    format!("{}...", &error_text[..200])
                } else {
                    error_text
                };
                trace_debug!("Error response: {}", preview);
            }

            // 重试，这次使用 Basic Auth
            let (email, api_token) = get_auth()?;
            let auth_retry = Authorization::new(email, api_token);
            let mut config_with_auth: RequestConfig<'_, serde_json::Value, serde_json::Value> =
                RequestConfig::new().timeout(std::time::Duration::from_secs(60));
            if !headers.is_empty() {
                config_with_auth = config_with_auth.headers(&headers);
            }
            config_with_auth = config_with_auth.auth(&auth_retry);

            response = client
                .stream(HttpMethod::Get, url, config_with_auth)
                .with_context(|| format!("Failed to download with Basic Auth: {}", url))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().unwrap_or_default();
                let error_msg = Self::format_download_error(status, error_text);
                anyhow::bail!("{}", error_msg);
            }
        } else if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            let error_msg = Self::format_download_error(status, error_text);
            anyhow::bail!("{}", error_msg);
        }

        // 写入文件
        let mut file = File::create(output_path)
            .with_context(|| format!("Failed to create file: {:?}", output_path))?;

        std::io::copy(&mut response, &mut file)
            .with_context(|| format!("Failed to write file: {:?}", output_path))?;

        Ok(())
    }

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
}
