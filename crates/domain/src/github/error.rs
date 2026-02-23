//! GitHub 错误类型

use thiserror::Error;

/// GitHub API 错误
#[derive(Error, Debug)]
pub enum GitHubError {
    #[error("The GitHub API call failed: {0}")]
    ApiError(String),

    #[error("The authentication failed")]
    AuthenticationFailed,

    #[error("The resource {0} is not found")]
    NotFound(String),

    #[error("The permissions are insufficient")]
    InsufficientPermissions,

    #[error("The rate limit exceeded: {0}")]
    RateLimitExceeded(String),
}

/// 从 GitHubClientError 转换为 GitHubError
impl From<client::GitHubClientError> for GitHubError {
    fn from(err: client::GitHubClientError) -> Self {
        GitHubError::ApiError(err.to_string())
    }
}
