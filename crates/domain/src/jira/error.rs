//! Jira 错误类型

use thiserror::Error;

/// Jira API 错误
#[derive(Error, Debug)]
pub enum JiraError {
    #[error("Jira API 调用失败: {0}")]
    ApiError(String),

    #[error("认证失败")]
    AuthenticationFailed,

    #[error("Issue 不存在: {0}")]
    IssueNotFound(String),

    #[error("项目不存在: {0}")]
    ProjectNotFound(String),

    #[error("状态转换无效")]
    InvalidTransition,

    #[error("验证错误: {0}")]
    ValidationError(String),

    #[error("其他错误: {0}")]
    Other(String),
}
