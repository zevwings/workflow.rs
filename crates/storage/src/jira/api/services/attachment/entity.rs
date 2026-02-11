//! Attachment 服务实体定义
//!
//! 提供 URL 解析和重试策略。

use std::{collections::HashMap, sync::Arc};

use domain::{JiraAttachment, JiraConfigContext, JiraIssue};
use regex::Regex;

/// URL 解析器
///
/// 提供多重 URL 重试策略，提高下载成功率。
pub struct UrlResolver {
    /// 原始 URL 映射（从 issue description 解析）
    original_urls: HashMap<String, String>,
    /// API 附件 URL 映射（从 API 获取）
    api_attachments_map: HashMap<String, String>,
    /// 配置上下文
    config_context: Arc<dyn JiraConfigContext>,
}

impl UrlResolver {
    /// 从 Jira ticket 创建 URL 解析器
    ///
    /// # 参数
    ///
    /// * `issue` - Jira Issue 信息
    /// * `config_context` - 配置上下文
    pub fn new(issue: &JiraIssue, config_context: Arc<dyn JiraConfigContext>) -> Self {
        let mut original_urls = HashMap::new();
        let mut api_attachments_map = HashMap::new();

        // 从 description 解析附件链接
        if let Some(description) = &issue.fields.description {
            for attachment in Self::parse_attachments_from_description(description) {
                original_urls.insert(attachment.filename.clone(), attachment.content_url);
            }
        }

        // 从 API 获取的附件列表
        if let Some(attachments) = &issue.fields.attachment {
            for attachment in attachments {
                api_attachments_map
                    .insert(attachment.filename.clone(), attachment.content_url.clone());
            }
        }

        Self {
            original_urls,
            api_attachments_map,
            config_context,
        }
    }

    /// 获取附件的所有可能下载 URL
    ///
    /// 返回多个 URL 用于重试，顺序为：
    /// 1. 原始 CloudFront URL（如果有）
    /// 2. API 附件 URL（如果有）
    /// 3. REST API attachment endpoint（如果能提取 attachment ID）
    /// 4. Description 中解析的原始 URL（如果有）
    ///
    /// # 参数
    ///
    /// * `attachment` - Jira 附件信息
    pub fn get_download_urls(&self, attachment: &JiraAttachment) -> Vec<String> {
        let mut urls = vec![attachment.content_url.clone()];

        // 如果是 CloudFront URL，添加备用 URL
        if attachment.content_url.contains("cloudfront.net") {
            // 1. 尝试从 API attachments map 获取
            if let Some(api_url) = self.api_attachments_map.get(&attachment.filename) {
                if api_url != &attachment.content_url {
                    urls.push(api_url.clone());
                }
            }

            // 2. 尝试构建 REST API URL
            if let Some(attachment_id) = Self::extract_attachment_id(&attachment.content_url) {
                if let Ok(base_url) = self.config_context.get_base_url() {
                    urls.push(format!(
                        "{}/rest/api/2/attachment/content/{}",
                        base_url, attachment_id
                    ));
                }
            }

            // 3. 尝试使用 description 中解析的原始 URL
            if let Some(original_url) = self.original_urls.get(&attachment.filename) {
                if original_url != &attachment.content_url && !urls.contains(original_url) {
                    urls.push(original_url.clone());
                }
            }
        }

        urls
    }

    /// 从 URL 中提取附件 ID
    ///
    /// CloudFront URL 格式通常为：
    /// `/attachments/bugs/16232306/.../21886523/log0.txt`
    ///
    /// 附件 ID 是路径中的数字部分（如 `21886523`）
    fn extract_attachment_id(url: &str) -> Option<String> {
        let id_pattern = Regex::new(r"/attachments/[^/]+/\d+/([^/]+/)*(\d+)/").ok()?;
        id_pattern.captures(url)?.get(2).map(|m| m.as_str().to_string())
    }

    /// 从描述中解析附件链接
    ///
    /// Jira 描述中可能包含附件链接，格式为：`# [filename|url]`
    fn parse_attachments_from_description(description: &str) -> Vec<JiraAttachment> {
        let mut attachments = Vec::new();
        let link_pattern = Regex::new(r#"#\s*\[([^|]+)\|([^\]]+)\]"#).ok();

        if let Some(pattern) = link_pattern {
            for cap in pattern.captures_iter(description) {
                if let (Some(filename_match), Some(url_match)) = (cap.get(1), cap.get(2)) {
                    let filename = filename_match.as_str().trim().to_string();
                    let url = url_match.as_str().trim().to_string();

                    if url.contains("attachments")
                        || filename.ends_with(".txt")
                        || filename.ends_with(".log")
                        || filename.ends_with(".zip")
                    {
                        attachments.push(JiraAttachment {
                            filename,
                            content_url: url,
                            mime_type: None,
                            size: None,
                        });
                    }
                }
            }
        }

        attachments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_attachment_id() {
        let url = "https://cdn.cloudfront.net/attachments/bugs/16232306/path/21886523/log0.txt";
        assert_eq!(
            UrlResolver::extract_attachment_id(url),
            Some("21886523".to_string())
        );
    }

    #[test]
    fn test_extract_attachment_id_no_match() {
        let url = "https://example.com/file.txt";
        assert_eq!(UrlResolver::extract_attachment_id(url), None);
    }

    #[test]
    fn test_parse_attachments_from_description() {
        let description = r#"
            Some text
            # [log.txt|https://example.com/attachments/log.txt]
            # [error.log|https://example.com/error.log]
            # [image.png|https://example.com/image.png]
        "#;

        let attachments = UrlResolver::parse_attachments_from_description(description);
        // 仅解析 .txt/.log/.zip 或 URL 含 attachments 的项，image.png 被过滤
        assert_eq!(attachments.len(), 2);
        assert_eq!(attachments[0].filename, "log.txt");
        assert_eq!(attachments[1].filename, "error.log");
    }
}
