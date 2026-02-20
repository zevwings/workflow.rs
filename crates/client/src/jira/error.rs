//! Jira 错误类型

use thiserror::Error;

/// Jira API 错误
#[derive(Error, Debug)]
pub enum JiraClientError {
    #[error("Jira API call failed: {0}")]
    ApiError(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Configuration error: {0}")]
    ConfigError(String),
}
