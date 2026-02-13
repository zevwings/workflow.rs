//! GitHub 错误类型

use thiserror::Error;

/// GitHub API 错误
#[derive(Error, Debug)]
pub enum GitHubClientError {
    #[error("GitHub API 调用失败: {0}")]
    ApiError(String),

    #[error("认证失败")]
    AuthenticationFailed,

    #[error("配置错误: {0}")]
    ConfigError(String),
}
