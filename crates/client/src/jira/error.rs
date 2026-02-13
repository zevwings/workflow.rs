//! Jira 错误类型

use thiserror::Error;

/// Jira API 错误
#[derive(Error, Debug)]
pub enum JiraClientError {
    #[error("Jira API 调用失败: {0}")]
    ApiError(String),

    #[error("认证失败")]
    AuthenticationFailed,

    #[error("配置错误: {0}")]
    ConfigError(String),
}
