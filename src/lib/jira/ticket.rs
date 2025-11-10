//! Jira Ticket/Issue 相关 API
//!
//! 本模块提供了 Ticket/Issue 的完整操作功能：
//! - 获取 ticket 信息（包括 summary、description、status、attachments、comments）
//! - 更新 ticket 状态（通过 transitions）
//! - 分配 ticket 给用户
//! - 添加评论到 ticket
//! - 获取 ticket 的附件列表

use anyhow::{Context, Result};
use serde::Serialize;

use crate::http::{Authorization, HttpClient};

use super::helpers::{get_auth, get_base_url};
use super::models::{JiraAttachment, JiraIssue, JiraTransition};

/// 获取 issue URL
///
/// 根据 ticket ID 构建完整的 issue API URL。
///
/// # 参数
///
/// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
///
/// # 返回
///
/// 返回完整的 issue API URL，格式：`{base_url}/issue/{ticket}`
fn get_issue_url(ticket: &str) -> Result<String> {
    let base_url = get_base_url()?;
    Ok(format!("{}/issue/{}", base_url, ticket))
}

/// 获取 ticket 信息
///
/// 从 Jira API 获取指定 ticket 的完整信息，包括：
/// - 基本信息（key、id、summary、description）
/// - 状态信息
/// - 附件列表
/// - 评论列表
///
/// # 参数
///
/// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
///
/// # 返回
///
/// 返回 `JiraIssue` 结构体，包含 ticket 的所有信息。
pub fn get_ticket_info(ticket: &str) -> Result<JiraIssue> {
    let (email, api_token) = get_auth()?;
    let mut url = get_issue_url(ticket)?;

    // 添加 fields 参数，明确请求所有需要的字段，包括附件
    // 使用 expand=renderedFields 可以获取更多信息
    url = format!("{}?fields=*all&expand=renderedFields", url);

    let client = HttpClient::new()?;
    let auth = Authorization::new(&email, &api_token);
    let response = client
        .get::<serde_json::Value>(&url, Some(&auth), None)
        .context(format!("Failed to get ticket info: {}", ticket))?;

    if !response.is_success() {
        anyhow::bail!("Failed to get ticket info: {}", response.status);
    }

    // 调试：打印附件信息
    if let Some(fields) = response.data.get("fields") {
        if let Some(attachments) = fields.get("attachment") {
            if let Some(attachments_array) = attachments.as_array() {
                crate::log_info!("Debug: Found {} attachments in API response", attachments_array.len());
                for (idx, att) in attachments_array.iter().enumerate() {
                    if let Some(filename) = att.get("filename").and_then(|f| f.as_str()) {
                        if let Some(content_url) = att.get("content").and_then(|c| c.as_str()) {
                            crate::log_info!("Debug: Attachment {}: {} -> {}", idx + 1, filename, content_url);
                        } else {
                            crate::log_info!("Debug: Attachment {}: {} (no content URL)", idx + 1, filename);
                        }
                    }
                }
            }
        } else {
            crate::log_info!("Debug: No 'attachment' field in API response");
        }
    }

    serde_json::from_value(response.data)
        .context(format!("Failed to parse ticket info for: {}", ticket))
}

/// 从 URL 中提取附件 ID
///
/// 从 CloudFront URL 中提取附件 ID，格式如：/attachments/bugs/16232306/.../21886523/log0.txt
/// 附件 ID 通常是路径中的数字部分。
pub fn extract_attachment_id_from_url(url: &str) -> Option<String> {
    use regex::Regex;
    // 匹配 URL 中的附件 ID，通常在路径中，如 /attachments/.../21886523/...
    let id_pattern = Regex::new(r"/attachments/[^/]+/\d+/([^/]+/)*(\d+)/").unwrap();
    if let Some(cap) = id_pattern.captures(url) {
        if let Some(id_match) = cap.get(2) {
            return Some(id_match.as_str().to_string());
        }
    }
    None
}

/// 从 CloudFront URL 构建 Jira API 的 content URL
///
/// 尝试从 CloudFront URL 中提取信息，构建 Jira API 的 content URL。
/// 格式：{base_url}/attachment/content/{attachment_id}
/// 注意：Jira API 的格式是 `/attachment/content/{id}`，不是 `/attachment/{id}/content`
#[allow(dead_code)]
fn build_jira_content_url_from_cloudfront(cloudfront_url: &str) -> Option<String> {
    // 从 CloudFront URL 中提取附件 ID
    if let Some(attachment_id) = extract_attachment_id_from_url(cloudfront_url) {
        // 尝试获取 base URL
        if let Ok(base_url) = get_base_url() {
            // 构建 Jira API 的 content URL
            // 格式：{base_url}/attachment/content/{attachment_id}
            // 注意：从 ticket info 获取的附件 URL 格式是：/rest/api/2/attachment/content/{id}
            let jira_url = format!("{}/attachment/content/{}", base_url, attachment_id);
            return Some(jira_url);
        }
    }
    None
}

/// 从描述中解析附件链接
///
/// Jira 描述中可能包含附件链接，格式为：`# [filename|url]`
/// 解析这些链接并返回附件列表。
fn parse_attachments_from_description(description: &str) -> Vec<JiraAttachment> {
    use regex::Regex;
    let mut attachments = Vec::new();

    // 匹配 Jira 链接格式：# [filename|url]
    // 例如：# [log0.txt|https://...]
    // 使用更宽松的匹配，允许 URL 中包含各种字符（包括查询参数）
    let link_pattern = Regex::new(r#"#\s*\[([^|]+)\|([^\]]+)\]"#).unwrap();

    for cap in link_pattern.captures_iter(description) {
        if let (Some(filename_match), Some(url_match)) = (cap.get(1), cap.get(2)) {
            let filename = filename_match.as_str().trim().to_string();
            let url = url_match.as_str().trim().to_string();

            // 只处理看起来像文件链接的 URL（包含 attachments 或 .txt/.log 等扩展名）
            if url.contains("attachments") ||
               filename.ends_with(".txt") ||
               filename.ends_with(".log") ||
               filename.ends_with(".zip") {
                crate::log_info!("Debug: Parsed attachment from description: {} -> {}", filename, url);

                // 尝试从 URL 中提取附件 ID
                if let Some(attachment_id) = extract_attachment_id_from_url(&url) {
                    crate::log_info!("Debug: Extracted attachment ID from URL: {}", attachment_id);
                }

                attachments.push(JiraAttachment {
                    filename,
                    content_url: url,
                    mime_type: None,
                    size: None,
                });
            }
        }
    }

    attachments
}

/// 通过附件 ID 获取附件的正确 URL
///
/// 从 Jira API 获取附件的正确下载 URL。
/// 附件 ID 可以通过 `/rest/api/2/attachment/{id}` 端点获取。
#[allow(dead_code)]
fn get_attachment_url_by_id(attachment_id: &str) -> Result<String> {
    let (email, api_token) = get_auth()?;
    let base_url = get_base_url()?;
    let url = format!("{}/attachment/{}", base_url, attachment_id);

    let client = HttpClient::new()?;
    let auth = Authorization::new(&email, &api_token);
    let response = client
        .get::<serde_json::Value>(&url, Some(&auth), None)
        .context(format!("Failed to get attachment info: {}", attachment_id))?;

    if !response.is_success() {
        anyhow::bail!("Failed to get attachment info: {}", response.status);
    }

    // 从响应中提取 content URL
    if let Some(content_url) = response.data.get("content").and_then(|c| c.as_str()) {
        Ok(content_url.to_string())
    } else {
        anyhow::bail!("No content URL found in attachment response")
    }
}

/// 获取 ticket 的附件列表
///
/// 用于日志下载等功能。
/// 会从 `fields.attachment` 获取附件，同时也会从描述中解析附件链接。
/// 对于从描述中解析出的 CloudFront URL，会尝试通过附件 ID 获取正确的 Jira API URL。
///
/// # 参数
///
/// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
///
/// # 返回
///
/// 返回附件列表，如果没有附件则返回空列表。
pub fn get_attachments(ticket: &str) -> Result<Vec<JiraAttachment>> {
    let issue = get_ticket_info(ticket)?;
    let mut attachments = issue.fields.attachment.unwrap_or_default();

    // 从描述中解析附件链接
    if let Some(description) = &issue.fields.description {
        let description_attachments = parse_attachments_from_description(description);

        // 合并附件，避免重复（基于文件名）
        for desc_att in description_attachments {
            // 检查是否已存在同名附件
            if !attachments.iter().any(|a| a.filename == desc_att.filename) {
                // 如果是 CloudFront 签名 URL，直接使用原始 URL
                // 注意：从 CloudFront URL 提取的 ID 通常不是真正的 Jira 附件 ID
                // 因此我们直接使用 CloudFront URL，让下载逻辑处理认证
                if desc_att.content_url.contains("cloudfront.net") {
                    crate::log_info!("Debug: Found CloudFront URL for {}: {}", desc_att.filename, desc_att.content_url);
                    // 保持原始 CloudFront URL，下载时会尝试不同的认证方式
                }

                crate::log_info!("Debug: Found attachment in description: {}", desc_att.filename);
                attachments.push(desc_att);
            }
        }
    }

    Ok(attachments)
}

/// 获取 issue 的可用 transitions
///
/// 获取指定 ticket 当前可以执行的所有状态转换。
///
/// # 参数
///
/// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
///
/// # 返回
///
/// 返回可用的 transitions 列表，每个 transition 包含 ID 和名称。
fn get_transitions(ticket: &str) -> Result<Vec<JiraTransition>> {
    let (email, api_token) = get_auth()?;
    let issue_url = get_issue_url(ticket)?;
    let url = format!("{}/transitions", issue_url);

    let client = HttpClient::new()?;
    let auth = Authorization::new(&email, &api_token);
    let response = client
        .get::<serde_json::Value>(&url, Some(&auth), None)
        .context(format!("Failed to get transitions for ticket: {}", ticket))?;

    if !response.is_success() {
        anyhow::bail!("Failed to get transitions: {}", response.status);
    }

    let transitions = response
        .data
        .get("transitions")
        .and_then(|t| t.as_array())
        .context("Invalid transitions JSON structure")?;

    let result: Vec<JiraTransition> = transitions
        .iter()
        .filter_map(|t| {
            let id = t.get("id")?.as_str()?.to_string();
            let name = t.get("name")?.as_str()?.to_string();
            Some(JiraTransition { id, name })
        })
        .collect();

    Ok(result)
}

/// 更新 ticket 状态
///
/// 将 ticket 的状态更新为指定的状态。
/// 会先获取 ticket 的可用 transitions，然后查找匹配的状态进行转换。
///
/// # 参数
///
/// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
/// * `status` - 目标状态名称，如 `"In Progress"`、`"Done"` 等
///
/// # 错误
///
/// 如果指定的状态不在可用 transitions 列表中，返回错误。
pub fn move_ticket(ticket: &str, status: &str) -> Result<()> {
    let (email, api_token) = get_auth()?;

    // 先获取可用的 transitions
    let transitions = get_transitions(ticket)?;

    // 查找匹配的 transition
    let transition = transitions
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(status))
        .context(format!(
            "Status '{}' not found in available transitions for ticket {}",
            status, ticket
        ))?;

    let issue_url = get_issue_url(ticket)?;
    let url = format!("{}/transitions", issue_url);

    #[derive(Serialize)]
    struct TransitionRequest {
        transition: TransitionRef,
    }

    #[derive(Serialize)]
    struct TransitionRef {
        id: String,
    }

    let body = TransitionRequest {
        transition: TransitionRef {
            id: transition.id.clone(),
        },
    };

    let client = HttpClient::new()?;
    let auth = Authorization::new(&email, &api_token);
    let response = client
        .post::<serde_json::Value, _>(&url, &body, Some(&auth), None)
        .context(format!(
            "Failed to move ticket {} to status {}",
            ticket, status
        ))?;

    if !response.is_success() {
        anyhow::bail!(
            "Failed to move ticket {} to status {}: {}",
            ticket,
            status,
            response.status
        );
    }

    Ok(())
}

/// 分配 ticket 给用户
///
/// 将 ticket 分配给指定的用户。如果 `assignee` 为 `None`，则分配给当前用户。
///
/// # 参数
///
/// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
/// * `assignee` - 被分配用户的 account_id，如果为 `None` 则分配给当前用户
pub fn assign_ticket(ticket: &str, assignee: Option<&str>) -> Result<()> {
    let (email, api_token) = get_auth()?;

    let assignee_account_id = match assignee {
        Some(user) => user.to_string(),
        None => {
            // 如果没有指定，分配给当前用户
            super::users::get_user_info()?.account_id
        }
    };

    let issue_url = get_issue_url(ticket)?;
    let url = format!("{}/assignee", issue_url);

    // 使用 serde 的 rename 属性将 Rust 的 snake_case 转换为 JSON 的 camelCase
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct AssigneeRequest {
        account_id: String,
    }

    let body = AssigneeRequest {
        account_id: assignee_account_id.clone(),
    };

    let client = HttpClient::new()?;
    let auth = Authorization::new(&email, &api_token);
    let response = client
        .put::<serde_json::Value, _>(&url, &body, Some(&auth), None)
        .context(format!(
            "Failed to assign ticket {} to {}",
            ticket, assignee_account_id
        ))?;

    if !response.is_success() {
        anyhow::bail!(
            "Failed to assign ticket {} to {}: {}",
            ticket,
            assignee_account_id,
            response.status
        );
    }

    Ok(())
}

/// 添加评论到 ticket
///
/// 在指定的 ticket 上添加一条评论。
///
/// # 参数
///
/// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
/// * `comment` - 评论内容
pub fn add_comment(ticket: &str, comment: &str) -> Result<()> {
    let (email, api_token) = get_auth()?;
    let issue_url = get_issue_url(ticket)?;
    let url = format!("{}/comment", issue_url);

    #[derive(Serialize)]
    struct CommentRequest {
        body: String,
    }

    let body = CommentRequest {
        body: comment.to_string(),
    };

    let client = HttpClient::new()?;
    let auth = Authorization::new(&email, &api_token);
    let response = client
        .post::<serde_json::Value, _>(&url, &body, Some(&auth), None)
        .context(format!("Failed to add comment to ticket {}", ticket))?;

    if !response.is_success() {
        anyhow::bail!(
            "Failed to add comment to ticket {}: {}",
            ticket,
            response.status
        );
    }

    Ok(())
}
