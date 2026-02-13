//! Pull Request 服务错误类型

use thiserror::Error;

use crate::git::GitError;
use crate::github::GitHubError;

/// Pull Request 服务错误
#[derive(Error, Debug)]
pub enum PullRequestError {
    #[error("Git 操作失败: {0}")]
    Git(String),

    #[error("GitHub 操作失败")]
    GitHub(#[from] GitHubError),

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("无效输入: {0}")]
    InvalidInput(String),

    #[error("不支持的操作: {0}")]
    UnsupportedOperation(String),

    #[error("{0}")]
    Other(String),
}

// 手动实现 GitError 到 PullRequestError 的转换
impl From<GitError> for PullRequestError {
    fn from(err: GitError) -> Self {
        PullRequestError::Git(err.to_string())
    }
}
