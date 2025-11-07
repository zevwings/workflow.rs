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
    let url = get_issue_url(ticket)?;

    let client = HttpClient::new()?;
    let auth = Authorization::new(&email, &api_token);
    let response = client
        .get::<serde_json::Value>(&url, Some(&auth), None)
        .context(format!("Failed to get ticket info: {}", ticket))?;

    if !response.is_success() {
        anyhow::bail!("Failed to get ticket info: {}", response.status);
    }

    serde_json::from_value(response.data)
        .context(format!("Failed to parse ticket info for: {}", ticket))
}

/// 获取 ticket 的附件列表
///
/// 用于日志下载等功能。
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
    Ok(issue.fields.attachment.unwrap_or_default())
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

