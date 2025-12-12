//! Jira 数据模型
//!
//! 本模块定义了所有与 Jira API 交互时使用的数据结构，
//! 包括 Issue、User、Attachment、Comment、Status 等。
//!
//! 所有结构体都实现了 `Serialize` 和 `Deserialize` trait，
//! 可以直接与 Jira API 的 JSON 格式进行序列化/反序列化。

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

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
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueFields {
    pub summary: String,
    pub description: Option<String>,
    pub status: JiraStatus,
    pub attachment: Option<Vec<JiraAttachment>>,
    pub comment: Option<JiraComments>,
    pub priority: Option<JiraPriority>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub reporter: Option<JiraUser>,
    pub assignee: Option<JiraUser>,
    pub labels: Option<Vec<String>>,
    pub components: Option<Vec<JiraComponent>>,
    pub fix_versions: Option<Vec<JiraVersion>>,
    pub issuelinks: Option<Vec<JiraIssueLink>>,
    pub subtasks: Option<Vec<JiraSubtask>>,
    pub time_tracking: Option<JiraTimeTracking>,
}

/// Jira 附件信息
///
/// 包含附件的文件名、内容 URL、MIME 类型和大小等信息。
#[skip_serializing_none]
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
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraComments {
    pub comments: Vec<JiraComment>,
    pub max_results: Option<u64>,
    pub start_at: Option<u64>,
    pub total: Option<u64>,
}

/// Jira 评论信息
///
/// 包含评论的 ID、内容、创建时间、更新时间、作者等信息。
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraComment {
    pub id: String,
    pub body: String,
    pub created: String,
    pub updated: Option<String>,
    pub author: Option<JiraUser>,
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

/// Jira 优先级信息
///
/// 包含优先级的 ID、名称和图标 URL。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraPriority {
    pub id: String,
    pub name: String,
    pub icon_url: Option<String>,
}

/// Jira 组件信息
///
/// 包含组件的 ID、名称和描述。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraComponent {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

/// Jira 版本信息
///
/// 包含版本的 ID、名称、发布状态和发布日期。
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraVersion {
    pub id: String,
    pub name: String,
    pub released: Option<bool>,
    pub release_date: Option<String>,
}

/// Jira Issue 链接信息
///
/// 包含关联的 Issue 信息和链接类型。
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueLink {
    pub id: String,
    pub link_type: Option<JiraIssueLinkType>,
    pub inward_issue: Option<JiraIssueRef>,
    pub outward_issue: Option<JiraIssueRef>,
}

/// Jira Issue 链接类型
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueLinkType {
    pub id: String,
    pub name: String,
    pub inward: Option<String>,
    pub outward: Option<String>,
}

/// Jira Issue 引用
///
/// 包含关联 Issue 的基本信息（key、id、summary）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueRef {
    pub key: String,
    pub id: String,
    pub fields: Option<JiraIssueRefFields>,
}

/// Jira Issue 引用字段
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueRefFields {
    pub summary: Option<String>,
    pub status: Option<JiraStatus>,
}

/// Jira 子任务信息
///
/// 包含子任务的 key、id 和基本信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraSubtask {
    pub key: String,
    pub id: String,
    pub fields: Option<JiraSubtaskFields>,
}

/// Jira 子任务字段
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraSubtaskFields {
    pub summary: Option<String>,
    pub status: Option<JiraStatus>,
}

/// Jira 时间跟踪信息
///
/// 包含原始估计时间、剩余时间和已用时间。
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraTimeTracking {
    pub original_estimate: Option<String>,
    pub remaining_estimate: Option<String>,
    pub time_spent: Option<String>,
    pub original_estimate_seconds: Option<i64>,
    pub remaining_estimate_seconds: Option<i64>,
    pub time_spent_seconds: Option<i64>,
}

/// Jira 变更历史
///
/// 包含 Issue 的所有变更记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraChangelog {
    pub id: String,
    pub histories: Vec<JiraChangelogHistory>,
}

/// Jira 变更历史记录
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraChangelogHistory {
    pub id: String,
    pub created: String,
    pub author: Option<JiraUser>,
    pub items: Vec<JiraChangelogItem>,
}

/// Jira 变更历史项
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraChangelogItem {
    pub field: String,
    pub field_type: Option<String>,
    pub field_id: Option<String>,
    pub from: Option<String>,
    pub from_string: Option<String>,
    pub to: Option<String>,
    pub to_string: Option<String>,
}
