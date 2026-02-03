//! Jira 实体类型和辅助函数

use serde::Serialize;

use crate::jira::error::JiraError;

/// Jira Issue 信息
#[derive(Debug, Clone, Serialize)]
pub struct JiraIssue {
    pub id: String,
    pub key: String,
    pub summary: String,
    pub status: String,
    pub assignee: Option<String>,
    pub description: Option<String>,
    pub attachments: Vec<JiraAttachment>,
    pub comments: Vec<JiraComment>,
    pub priority: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub reporter: Option<JiraUser>,
    pub labels: Vec<String>,
    pub components: Vec<String>,
}

/// Jira 用户信息
#[derive(Debug, Clone, Serialize)]
pub struct JiraUser {
    pub display_name: String,
    pub account_id: String,
}

/// Jira 评论信息
#[derive(Debug, Clone, Serialize)]
pub struct JiraComment {
    pub id: String,
    pub body: String,
    pub created: String,
    pub author: Option<JiraUser>,
}

/// Jira 附件信息
#[derive(Debug, Clone, Serialize)]
pub struct JiraAttachment {
    pub id: String,
    pub filename: String,
    pub size: u64,
    pub url: String,
}

// ============================================================================
// 辅助函数
// ============================================================================

// 验证错误消息常量
/// 无效的 JIRA ID 格式
const VALIDATION_INVALID_JIRA_ID_FORMAT: &str = "Invalid JIRA ID format";

/// JIRA ID 格式说明
const VALIDATION_JIRA_ID_FORMAT_HELP: &str = "Expected formats:\n\
    • Ticket ID: PROJ-123 (project code + hyphen + number)\n\
    • Project name: PROJ (letters, numbers, underscores only)";

/// JIRA ID 不能为空
const VALIDATION_JIRA_ID_EMPTY: &str = "JIRA ID cannot be empty";

/// 从 Jira ticket 提取项目名
///
/// # 示例
/// ```
/// use domain::jira::extract_jira_project;
/// assert_eq!(extract_jira_project("PROJ-123"), Some("PROJ"));
/// assert_eq!(extract_jira_project("PROJ"), None);
/// ```
pub fn extract_jira_project(ticket: &str) -> Option<&str> {
    ticket.split('-').next().filter(|s| *s != ticket)
}

/// 验证 Jira ticket 格式
///
/// Jira ticket 应该是 PROJECT-123 格式（ticket），或纯项目名（PROJECT）。
/// 项目名只能包含字母、数字和下划线。
///
/// # 示例
/// ```
/// use domain::jira::validate_jira_ticket_format;
/// assert!(validate_jira_ticket_format("PROJ-123").is_ok());
/// assert!(validate_jira_ticket_format("PROJ").is_ok());
/// assert!(validate_jira_ticket_format("PROJ-123-456").is_ok());
/// assert!(validate_jira_ticket_format("invalid/ticket").is_err());
/// ```
pub fn validate_jira_ticket_format(ticket: &str) -> Result<(), JiraError> {
    // 先检查是否为空或只包含空白字符
    if ticket.trim().is_empty() {
        return Err(JiraError::ValidationError(format!(
            "{}: {}",
            VALIDATION_INVALID_JIRA_ID_FORMAT, VALIDATION_JIRA_ID_EMPTY
        )));
    }

    let is_valid_format: bool = if let Some(project) = extract_jira_project(ticket) {
        // 如果是 ticket 格式（PROJ-123），需要：
        // 1. 项目名有效（只包含字母、数字、下划线）
        // 2. ticket 必须包含数字部分（PROJ-123 格式，不能只是 PROJ-）
        let project_valid = project.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');

        // 检查是否有数字部分（ticket 格式应该是 PROJECT-NUMBER）
        let has_number_part =
            ticket.split('-').skip(1).any(|part| part.chars().any(|c| c.is_ascii_digit()));

        project_valid && has_number_part
    } else {
        // 如果是项目名格式，检查是否只包含有效字符
        ticket.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    };

    if !is_valid_format {
        return Err(JiraError::ValidationError(format!(
            "{}: '{}'. {}\n  - Ticket names should contain only letters, numbers, and hyphens\n  - Project names should contain only letters, numbers, and underscores\n  - Do not use branch names or paths (e.g., 'zw/修改打包脚本问题')",
            VALIDATION_INVALID_JIRA_ID_FORMAT, ticket, VALIDATION_JIRA_ID_FORMAT_HELP
        )));
    }

    Ok(())
}
