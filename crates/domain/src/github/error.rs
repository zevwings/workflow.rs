//! GitHub 错误类型

use thiserror::Error;

/// GitHub API 错误
#[derive(Error, Debug)]
pub enum GitHubError {
    #[error("GitHub API 调用失败: {0}")]
    ApiError(String),

    #[error("认证失败")]
    AuthenticationFailed,

    #[error("资源不存在: {0}")]
    NotFound(String),

    #[error("权限不足")]
    InsufficientPermissions,

    #[error("速率限制: {0}")]
    RateLimitExceeded(String),

    #[error("配置错误: {0}")]
    ConfigError(String),
}
