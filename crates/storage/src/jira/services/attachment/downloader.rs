// use crate::jira::types::JiraAttachment;
// use color_eyre::{eyre::eyre, eyre::WrapErr, Result};

// use domain::jira::JiraConfigContext;
// use reqwest::header::HeaderMap;
// use std::fs::File;
// use std::path::{Path, PathBuf};
// use std::sync::Arc;
// use toolkit::log_debug;
// use toolkit::{Authorization, HttpClient, HttpMethod, RequestConfig};

// /// 下载结果类型别名
// type DownloadResult = (Vec<PathBuf>, Vec<(String, String)>);

// pub struct Downloader {
//     config_context: Arc<dyn JiraConfigContext>,
// }

// impl Downloader {
//     pub fn new(config_context: Arc<dyn JiraConfigContext>) -> Self {
//         Self { config_context }
//     }

//     /// 尝试下载文件
//     pub fn try_download_file(
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
//     pub fn download_file(&self, url: &str, output_path: &Path) -> Result<()> {
//         let client = HttpClient::global()?;
//         let mut headers = HeaderMap::new();
//         if let Ok(base_url) = self.config_context.get_base_url() {
//             if let Ok(referer) = base_url.parse() {
//                 headers.insert("Referer", referer);
//             }
//         }

//         let is_cloudfront = url.contains("cloudfront.net")
//             && url.contains("Expires=")
//             && url.contains("Signature=");
//         let mut config = RequestConfig::new().timeout(std::time::Duration::from_secs(60));
//         if !headers.is_empty() {
//             config = config.headers(&headers);
//         }

//         if !is_cloudfront {
//             let (email, api_token) = self.config_context.get_auth()?;
//             config = config.auth(Authorization::basic(email, api_token));
//         }

//         let mut response = client
//             .stream(HttpMethod::Get, url, config)
//             .wrap_err_with(|| format!("Failed to download: {}", url))?;

//         if !response.status().is_success() && is_cloudfront {
//             log_debug!("CloudFront URL failed, retrying with Basic Auth");
//             let (email, api_token) = self.config_context.get_auth()?;
//             let mut config = RequestConfig::new().timeout(std::time::Duration::from_secs(60));
//             if !headers.is_empty() {
//                 config = config.headers(&headers);
//             }
//             config = config.auth(Authorization::basic(email, api_token));

//             response = client
//                 .stream(HttpMethod::Get, url, config)
//                 .wrap_err_with(|| format!("Failed to download with Basic Auth: {}", url))?;

//             if !response.status().is_success() {
//                 let status = response.status();
//                 let error_text = response.text().unwrap_or_default();
//                 let preview = if error_text.len() > 200 {
//                     format!("{}...", &error_text[..200])
//                 } else {
//                     error_text
//                 };
//                 color_eyre::eyre::bail!("Download failed with status: {} - {}", status, preview);
//             }
//         } else if !response.status().is_success() {
//             let status = response.status();
//             let error_text = response.text().unwrap_or_default();
//             let preview = if error_text.len() > 200 {
//                 format!("{}...", &error_text[..200])
//             } else {
//                 error_text
//             };
//             color_eyre::eyre::bail!("Download failed with status: {} - {}", status, preview);
//         }

//         let mut file = File::create(output_path)
//             .wrap_err_with(|| format!("Failed to create file: {:?}", output_path))?;
//         std::io::copy(&mut response, &mut file)
//             .wrap_err_with(|| format!("Failed to write file: {:?}", output_path))?;

//         Ok(())
//     }
// }
