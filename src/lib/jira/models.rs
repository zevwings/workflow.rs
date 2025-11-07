//! Jira 数据模型
//!
//! 本模块定义了所有与 Jira API 交互时使用的数据结构，
//! 包括 Issue、User、Attachment、Comment、Status 等。
//!
//! 所有结构体都实现了 `Serialize` 和 `Deserialize` trait，
//! 可以直接与 Jira API 的 JSON 格式进行序列化/反序列化。

use serde::{Deserialize, Serialize};

/// Jira Issue 完整信息
///
/// 包含 Issue 的基本信息和所有字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssue {
    pub key: String,
    pub id: String,
    #[serde(rename = "self")]
    pub self_url: String,
    pub fields: JiraIssueFields,
}

/// Jira Issue 字段
///
/// 包含 Issue 的所有字段信息，如 summary、description、status、attachment、comment 等。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueFields {
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: JiraStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<Vec<JiraAttachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<JiraComments>,
}

/// Jira 附件信息
///
/// 包含附件的文件名、内容 URL、MIME 类型和大小等信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraAttachment {
    pub filename: String,
    #[serde(rename = "content")]
    pub content_url: String,
    pub mime_type: Option<String>,
    pub size: Option<u64>,
}

/// Jira 评论容器
///
/// 包含评论列表以及分页信息（max_results、start_at、total）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraComments {
    pub comments: Vec<JiraComment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

/// Jira 评论信息
///
/// 包含评论的 ID、内容、创建时间、更新时间、作者等信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraComment {
    pub id: String,
    pub body: String,
    pub created: String,
    pub updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<JiraUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_author: Option<JiraUser>,
}

/// Jira 状态信息
///
/// 包含状态的 ID、名称和 URL。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraStatus {
    pub id: String,
    pub name: String,
    #[serde(rename = "self")]
    pub self_url: Option<String>,
}

/// Jira 用户信息
///
/// 包含用户的 account_id、display_name 和 email_address。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraUser {
    pub account_id: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_address: Option<String>,
}

/// Jira Transition 信息
///
/// 用于状态转换，包含 transition 的 ID 和名称。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraTransition {
    pub id: String,
    pub name: String,
}
