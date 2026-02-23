//! Jira 业务域
//!
//! 包含 Jira 相关的实体、仓储接口和错误类型

pub mod api;
pub mod error;
pub mod history;

pub use api::{
    extract_jira_project, extract_jira_ticket_id, validate_jira_ticket_format,
    AttachmentDownloadResult, JiraAttachment, JiraComment, JiraComponent, JiraIssue, JiraPriority,
    JiraRepository, JiraStatusConfig, JiraTransition, JiraUser, ProgressCallback,
    ProjectStatusConfig, StatusConfigResult,
};

pub use error::JiraError;
pub use history::{DeleteHistoryResult, JiraWorkHistoryRepository, WorkHistoryEntry};
