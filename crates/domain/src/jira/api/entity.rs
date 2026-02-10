//! Jira 实体类型和辅助函数

use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::JiraError;

/// 预编译的 Jira ticket 正则表达式
///
/// 匹配格式：PROJECT-NUMBER（如 `PROJ-123`、`MY_PROJECT-456`）
static JIRA_TICKET_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([A-Z][A-Z0-9_]*-\d+)").expect("Invalid regex pattern"));
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

// pub struct JiraComment {
//     pub id: String,
//     pub body: String,
//     pub created: String,
//     pub updated: Option<String>,
//     pub author: Option<JiraUser>,
//     pub update_author: Option<JiraUser>,
// }

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

// /// Jira 评论信息
// #[derive(Debug, Clone, Serialize)]
// pub struct JiraComment {
//     pub id: String,
//     pub body: String,
//     pub created: String,
//     pub author: Option<JiraUser>,
// }

// /// Jira 附件信息
// #[derive(Debug, Clone, Serialize)]
// pub struct JiraAttachment {
//     pub id: String,
//     pub filename: String,
//     pub size: u64,
//     pub url: String,
// }

/// 状态配置结果
#[derive(Debug, Clone)]
pub struct StatusConfigResult {
    /// 项目名称
    pub project: String,
    /// PR 创建时的目标状态
    pub created_pull_request_status: String,
    /// PR 合并时的目标状态
    pub merged_pull_request_status: String,
}

/// 项目状态配置
///
/// 存储单个项目的状态配置，包括 PR 创建和合并时的目标状态。
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatusConfig {
    /// PR 创建时的目标状态（JSON 字段名：`created-pr`）
    #[serde(rename = "created-pr")]
    pub created_pull_request_status: Option<String>,
    /// PR 合并时的目标状态（JSON 字段名：`merged-pr`）
    #[serde(rename = "merged-pr")]
    pub merged_pull_request_status: Option<String>,
}

/// Jira 状态配置
///
/// 包含项目名称和对应的状态配置。
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatusConfig {
    /// 项目名称（如 `"PROJ"`）
    pub project: String,
    /// PR 创建时的目标状态
    pub created_pull_request_status: Option<String>,
    /// PR 合并时的目标状态
    pub merged_pull_request_status: Option<String>,
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
/// use domain::extract_jira_project;
/// assert_eq!(extract_jira_project("PROJ-123"), Some("PROJ"));
/// assert_eq!(extract_jira_project("PROJ"), None);
/// ```
pub fn extract_jira_project(ticket: &str) -> Option<&str> {
    ticket.split('-').next().filter(|s| *s != ticket)
}

/// 从文本中提取 Jira ticket ID
///
/// 支持从各种文本格式中提取 Jira ticket ID，例如：
/// - PR 标题：`"PROJ-123: Fix bug"` → `Some("PROJ-123")`
/// - Commit 消息：`"feat(scope): PROJ-456 add feature"` → `Some("PROJ-456")`
/// - 分支名：`"feature/PROJ-789-add-feature"` → `Some("PROJ-789")`
/// - 纯文本：`"This is about PROJ-111"` → `Some("PROJ-111")`
///
/// # 示例
/// ```
/// use domain::extract_jira_ticket_id;
/// assert_eq!(extract_jira_ticket_id("PROJ-123: Fix bug"), Some("PROJ-123".to_string()));
/// assert_eq!(extract_jira_ticket_id("No ticket here"), None);
/// assert_eq!(extract_jira_ticket_id("feature/ABC-456-test"), Some("ABC-456".to_string()));
/// ```
pub fn extract_jira_ticket_id(text: &str) -> Option<String> {
    JIRA_TICKET_REGEX.captures(text).map(|c| c[1].to_string())
}

/// 验证 Jira ticket 格式
///
/// Jira ticket 应该是 PROJECT-123 格式（ticket），或纯项目名（PROJECT）。
/// 项目名只能包含字母、数字和下划线。
///
/// # 示例
/// ```
/// use domain::validate_jira_ticket_format;
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

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // extract_jira_project 测试
    // ========================================================================

    #[test]
    fn test_extract_jira_project_valid_ticket() {
        assert_eq!(extract_jira_project("PROJ-123"), Some("PROJ"));
        assert_eq!(extract_jira_project("ABC-1"), Some("ABC"));
        assert_eq!(extract_jira_project("TEST-99999"), Some("TEST"));
    }

    #[test]
    fn test_extract_jira_project_complex_ticket() {
        // 多段格式（如 PROJ-123-456）
        assert_eq!(extract_jira_project("PROJ-123-456"), Some("PROJ"));
    }

    #[test]
    fn test_extract_jira_project_no_hyphen() {
        // 没有连字符，返回 None
        assert_eq!(extract_jira_project("PROJ"), None);
        assert_eq!(extract_jira_project("PROJECT123"), None);
    }

    #[test]
    fn test_extract_jira_project_empty() {
        assert_eq!(extract_jira_project(""), None);
    }

    #[test]
    fn test_extract_jira_project_only_hyphen() {
        // 只有连字符开头
        assert_eq!(extract_jira_project("-123"), Some(""));
    }

    // ========================================================================
    // validate_jira_ticket_format 测试
    // ========================================================================

    #[test]
    fn test_validate_jira_ticket_format_valid_ticket() {
        // 有效的 ticket 格式
        assert!(validate_jira_ticket_format("PROJ-123").is_ok());
        assert!(validate_jira_ticket_format("ABC-1").is_ok());
        assert!(validate_jira_ticket_format("TEST-99999").is_ok());
        assert!(validate_jira_ticket_format("MY_PROJECT-100").is_ok());
    }

    #[test]
    fn test_validate_jira_ticket_format_valid_project() {
        // 有效的项目名格式
        assert!(validate_jira_ticket_format("PROJ").is_ok());
        assert!(validate_jira_ticket_format("MY_PROJECT").is_ok());
        assert!(validate_jira_ticket_format("Test123").is_ok());
    }

    #[test]
    fn test_validate_jira_ticket_format_multi_segment() {
        // 多段 ticket 格式
        assert!(validate_jira_ticket_format("PROJ-123-456").is_ok());
    }

    #[test]
    fn test_validate_jira_ticket_format_empty() {
        // 空字符串
        let result = validate_jira_ticket_format("");
        assert!(result.is_err());

        // 只有空白字符
        let result = validate_jira_ticket_format("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_jira_ticket_format_invalid_chars() {
        // 包含无效字符
        assert!(validate_jira_ticket_format("invalid/ticket").is_err());
        assert!(validate_jira_ticket_format("PROJ@123").is_err());
        assert!(validate_jira_ticket_format("test#project").is_err());
    }

    #[test]
    fn test_validate_jira_ticket_format_no_number() {
        // ticket 格式但没有数字部分
        assert!(validate_jira_ticket_format("PROJ-ABC").is_err());
        assert!(validate_jira_ticket_format("PROJ-").is_err());
    }

    #[test]
    fn test_validate_jira_ticket_format_branch_like() {
        // 类似分支名的格式（应该失败）
        assert!(validate_jira_ticket_format("zw/修改打包脚本问题").is_err());
        assert!(validate_jira_ticket_format("feature/test").is_err());
    }

    // ========================================================================
    // extract_jira_ticket_id 测试
    // ========================================================================

    #[test]
    fn test_extract_jira_ticket_id_from_pr_title() {
        // PR 标题格式
        assert_eq!(
            extract_jira_ticket_id("PROJ-123: Fix bug"),
            Some("PROJ-123".to_string())
        );
        assert_eq!(
            extract_jira_ticket_id("PROJ-456 - Add feature"),
            Some("PROJ-456".to_string())
        );
    }

    #[test]
    fn test_extract_jira_ticket_id_from_commit_message() {
        // Commit 消息格式
        assert_eq!(
            extract_jira_ticket_id("feat(scope): PROJ-789 add feature"),
            Some("PROJ-789".to_string())
        );
        assert_eq!(
            extract_jira_ticket_id("[ABC-111] fix: resolve issue"),
            Some("ABC-111".to_string())
        );
    }

    #[test]
    fn test_extract_jira_ticket_id_from_branch_name() {
        // 分支名格式
        assert_eq!(
            extract_jira_ticket_id("feature/PROJ-222-add-feature"),
            Some("PROJ-222".to_string())
        );
        assert_eq!(
            extract_jira_ticket_id("bugfix/ABC-333-fix-bug"),
            Some("ABC-333".to_string())
        );
    }

    #[test]
    fn test_extract_jira_ticket_id_no_match() {
        // 无匹配
        assert_eq!(extract_jira_ticket_id("No ticket here"), None);
        assert_eq!(extract_jira_ticket_id("lowercase-123"), None);
        assert_eq!(extract_jira_ticket_id("PROJ without number"), None);
        assert_eq!(extract_jira_ticket_id(""), None);
    }

    #[test]
    fn test_extract_jira_ticket_id_first_match() {
        // 多个匹配时返回第一个
        assert_eq!(
            extract_jira_ticket_id("PROJ-111 and PROJ-222"),
            Some("PROJ-111".to_string())
        );
    }

    #[test]
    fn test_extract_jira_ticket_id_with_underscore() {
        // 项目名包含下划线
        assert_eq!(
            extract_jira_ticket_id("MY_PROJECT-123: test"),
            Some("MY_PROJECT-123".to_string())
        );
    }
}
