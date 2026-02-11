//! 通用服务错误

use thiserror::Error;

use crate::{GitError, GitHubError, JiraError};

/// 通用服务错误
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Git 错误: {0}")]
    Git(#[from] GitError),

    #[error("GitHub 错误: {0}")]
    GitHub(#[from] GitHubError),

    #[error("Jira 错误: {0}")]
    Jira(#[from] JiraError),

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_error_not_found_display() {
        let error = ServiceError::NotFound("资源不存在".to_string());
        assert_eq!(error.to_string(), "资源不存在");
    }

    #[test]
    fn test_service_error_unsupported_operation_display() {
        let error = ServiceError::UnsupportedOperation("暂不支持此功能".to_string());
        assert_eq!(error.to_string(), "不支持的操作: 暂不支持此功能");
    }

    #[test]
    fn test_service_error_invalid_input_display() {
        let error = ServiceError::InvalidInput("参数格式错误".to_string());
        assert_eq!(error.to_string(), "无效输入: 参数格式错误");
    }

    #[test]
    fn test_service_error_validation_failed_display() {
        let error = ServiceError::ValidationFailed("字段校验失败".to_string());
        assert_eq!(error.to_string(), "验证失败: 字段校验失败");
    }

    #[test]
    fn test_service_error_operation_failed_display() {
        let error = ServiceError::OperationFailed("执行过程中发生错误".to_string());
        assert_eq!(error.to_string(), "操作失败: 执行过程中发生错误");
    }

    #[test]
    fn test_service_error_other_display() {
        let error = ServiceError::Other("未知错误".to_string());
        assert_eq!(error.to_string(), "其他错误: 未知错误");
    }

    #[test]
    fn test_service_error_from_git_error() {
        let git_error = GitError::RepositoryNotFound("仓库不存在".to_string());
        let service_error: ServiceError = git_error.into();
        assert!(matches!(service_error, ServiceError::Git(_)));
        assert!(service_error.to_string().contains("Git 错误"));
    }

    #[test]
    fn test_service_error_from_github_error() {
        let github_error = GitHubError::AuthenticationFailed;
        let service_error: ServiceError = github_error.into();
        assert!(matches!(service_error, ServiceError::GitHub(_)));
        assert!(service_error.to_string().contains("GitHub 错误"));
    }

    #[test]
    fn test_service_error_from_jira_error() {
        let jira_error = JiraError::AuthenticationFailed;
        let service_error: ServiceError = jira_error.into();
        assert!(matches!(service_error, ServiceError::Jira(_)));
        assert!(service_error.to_string().contains("Jira 错误"));
    }

    #[test]
    fn test_service_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        // ServiceError 应该是 Send + Sync 的（如果底层错误都是）
        // 这个测试确保错误类型可以安全地跨线程传递
        assert_send_sync::<ServiceError>();
    }
}
