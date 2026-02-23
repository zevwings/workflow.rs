//! Pull Request 服务错误类型

use thiserror::Error;

use crate::git::GitError;
use crate::github::GitHubError;

/// Pull Request 服务错误
#[derive(Error, Debug)]
pub enum PullRequestError {
    #[error("The Pull Request ID is invalid: {0}")]
    InvalidPullRequestId(String),

    #[error("The Git operation failed: {0}")]
    Git(String),

    #[error("The GitHub operation failed")]
    GitHub(#[from] GitHubError),

    #[error("The {0} is not found")]
    NotFound(String),

    #[error("The input is invalid: {0}")]
    InvalidInput(String),

    #[error("The operation is not supported: {0}")]
    UnsupportedOperation(String),

    #[error("Other error: {0}")]
    Other(String),
}

// 手动实现 GitError 到 PullRequestError 的转换
impl From<GitError> for PullRequestError {
    fn from(err: GitError) -> Self {
        PullRequestError::Git(err.to_string())
    }
}
