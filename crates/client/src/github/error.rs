//! GitHub 错误类型

use thiserror::Error;

/// GitHub API 错误
#[derive(Error, Debug)]
pub enum GitHubClientError {
    #[error("GitHub API call failed: {0}")]
    ApiError(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Configuration error: {0}")]
    ConfigError(String),
}
