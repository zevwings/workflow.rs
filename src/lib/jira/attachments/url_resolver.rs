//! URL 解析和重试策略

use crate::jira::helpers::get_base_url;
use crate::jira::ticket::JiraTicket;
use crate::Jira;
use crate::JiraAttachment;
use color_eyre::Result;
use regex::Regex;
use std::collections::HashMap;

/// URL 解析器
///
/// 从 Jira ticket 中提取 URL 映射，并提供多种下载 URL 策略。
pub struct UrlResolver {
    /// 从 ticket 描述中提取的原始 CloudFront URL 映射
    original_urls: HashMap<String, String>,
    /// 从 API 附件列表中提取的 URL 映射
    api_attachments_map: HashMap<String, String>,
}

impl UrlResolver {
    /// 从 ticket 提取 URL 映射
    ///
    /// 从 Jira ticket 的描述和附件列表中提取 URL 映射，用于重试逻辑。
    ///
    /// # 参数
    ///
    /// * `jira_id` - Jira ticket ID
    ///
    /// # 返回
    ///
    /// 返回 `UrlResolver` 实例，包含提取的 URL 映射。
    pub fn from_ticket(jira_id: &str) -> Result<Self> {
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

        Ok(Self {
            original_urls,
            api_attachments_map,
        })
    }

    /// 获取所有可能的下载 URL（按优先级排序）
    ///
    /// 根据附件信息和 URL 映射，生成所有可能的下载 URL 列表。
    /// URL 按优先级排序，第一个是原始 URL，后续是备选 URL。
    ///
    /// # 参数
    ///
    /// * `attachment` - 附件信息
    ///
    /// # 返回
    ///
    /// 返回 URL 列表，按优先级排序。
    pub fn get_download_urls(&self, attachment: &JiraAttachment) -> Vec<String> {
        let mut urls = vec![attachment.content_url.clone()];

        // 如果当前 URL 是 CloudFront URL，尝试多种方式
        if attachment.content_url.contains("cloudfront.net") {
            // 方式 1: 从 API 附件列表中查找（如果提供了映射）
            if let Some(api_url) = self.api_attachments_map.get(&attachment.filename) {
                urls.push(api_url.clone());
            }

            // 方式 2: 从 CloudFront URL 中提取附件 ID 并构建 Jira API URL
            if let Some(attachment_id) =
                JiraTicket::extract_attachment_id_from_url(&attachment.content_url)
            {
                if let Ok(base_url) = get_base_url() {
                    urls.push(format!("{}/attachment/content/{}", base_url, attachment_id));
                }
            }

            // 方式 3: 原始 CloudFront URL（如果不同且提供了映射）
            if let Some(original_url) = self.original_urls.get(&attachment.filename) {
                if original_url != &attachment.content_url {
                    urls.push(original_url.clone());
                }
            }
        }

        urls
    }

    /// 获取原始 URL 映射（用于调试）
    #[allow(dead_code)]
    pub fn original_urls(&self) -> &HashMap<String, String> {
        &self.original_urls
    }

    /// 获取 API 附件映射（用于调试）
    #[allow(dead_code)]
    pub fn api_attachments_map(&self) -> &HashMap<String, String> {
        &self.api_attachments_map
    }
}
