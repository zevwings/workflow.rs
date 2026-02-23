//! Jira 错误类型

use thiserror::Error;

/// Jira API 错误
#[derive(Error, Debug)]
pub enum JiraError {
    #[error("The Jira API call failed: {0}")]
    ApiError(String),

    #[error("The issue {0} is not found")]
    IssueNotFound(String),

    #[error("The project {0} is not found")]
    ProjectNotFound(String),

    #[error("The status transition is invalid")]
    InvalidTransition,

    #[error("The validation error: {0}")]
    ValidationError(String),

    #[error("The IO error: {0}")]
    IoError(String),

    #[error("The network error: {0}")]
    NetworkError(String),

    #[error("The configuration error: {0}")]
    ConfigError(String),
}

/// 从 JiraClientError 转换为 JiraError
impl From<client::JiraClientError> for JiraError {
    fn from(err: client::JiraClientError) -> Self {
        JiraError::ApiError(err.to_string())
    }
}
