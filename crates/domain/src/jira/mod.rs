//! Jira 业务域
//!
//! 包含 Jira 相关的实体、仓储接口和错误类型

pub mod context;
pub mod entity;
pub mod error;
pub mod repository;
pub mod status;

// Re-export public types
pub use context::JiraConfigContext;
pub use entity::{
    extract_jira_project, validate_jira_ticket_format, JiraAttachment, JiraComment, JiraIssue,
    JiraUser,
};
pub use error::JiraError;
pub use repository::{AttachmentDownloadResult, JiraRepository};
pub use status::{JiraStatusConfig, ProjectStatusConfig, StatusConfigResult};
