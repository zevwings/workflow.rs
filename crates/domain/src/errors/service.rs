//! 通用服务错误

use crate::git::error::GitError;
use crate::github::error::GitHubError;
use crate::jira::error::JiraError;
use crate::llm::error::LLMError;
use thiserror::Error;

/// 通用服务错误
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Git 错误: {0}")]
    Git(#[from] GitError),

    #[error("GitHub 错误: {0}")]
    GitHub(#[from] GitHubError),

    #[error("Jira 错误: {0}")]
    Jira(#[from] JiraError),

    #[error("LLM 错误: {0}")]
    LLM(#[from] LLMError),

    #[error("{0}")]
    NotFound(String),

    #[error("不支持的操作: {0}")]
    UnsupportedOperation(String),

    #[error("无效输入: {0}")]
    InvalidInput(String),

    #[error("验证失败: {0}")]
    ValidationFailed(String),

    #[error("操作失败: {0}")]
    OperationFailed(String),

    #[error("其他错误: {0}")]
    Other(String),
}
