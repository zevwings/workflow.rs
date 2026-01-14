//! 验证错误消息

/// 无效的 PR 编号
pub const VALIDATION_INVALID_PR_NUMBER: &str = "Invalid PR number";

/// 无效的仓库格式
pub const VALIDATION_INVALID_REPO_FORMAT: &str = "Invalid repo format";

/// 无效的 JIRA ID 格式
pub const VALIDATION_INVALID_JIRA_ID_FORMAT: &str = "Invalid JIRA ID format";

/// JIRA ID 格式说明
pub const VALIDATION_JIRA_ID_FORMAT_HELP: &str = "Expected formats:\n\
    • Ticket ID: PROJ-123 (project code + hyphen + number)\n\
    • Project name: PROJ (letters, numbers, underscores only)";

/// JIRA ID 不能为空
pub const VALIDATION_JIRA_ID_EMPTY: &str = "JIRA ID cannot be empty";

/// JIRA ID 格式验证失败的完整消息模板
pub const VALIDATION_JIRA_ID_VALIDATION_ERROR_TEMPLATE: &str =
    "Invalid JIRA ID format.\n{}\n\nError details: {}";
