//! Codeup 错误类型

use thiserror::Error;

/// Codeup API 错误
#[derive(Error, Debug)]
pub enum CodeupError {
    #[error("Codeup API 调用失败: {0}")]
    ApiError(String),

    #[error("认证失败，请检查 CSRF Token 和 Cookie 是否有效")]
    AuthenticationFailed,

    #[error("资源不存在: {0}")]
    NotFound(String),

    #[error("权限不足")]
    InsufficientPermissions,

    #[error("配置不完整，请检查 codeup.project_id、codeup.csrf_token 和 codeup.cookie")]
    ConfigurationIncomplete,

    #[error("PR 已存在: {0}")]
    PullRequestAlreadyExists(String),

    #[error("PR 无法合并: {0}")]
    PullRequestNotMergeable(String),
}

/// 从 CodeupClientError 转换为 CodeupError
impl From<client::CodeupClientError> for CodeupError {
    fn from(err: client::CodeupClientError) -> Self {
        CodeupError::ApiError(err.to_string())
    }
}
