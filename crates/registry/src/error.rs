//! 依赖注入错误类型

use thiserror::Error;

/// 依赖注入错误
#[derive(Error, Debug)]
pub enum RegistryError {
    /// 服务未绑定
    #[error("Service not bound: {0}")]
    NotBound(String),

    /// 循环依赖
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// 类型转换错误
    #[error("Type cast error: {0}")]
    TypeCast(String),

    /// 锁错误（保留用于并发场景）
    #[allow(dead_code)]
    #[error("Lock error: {0}")]
    LockError(String),

    /// 服务已绑定
    #[error("Service already bound: {0}")]
    AlreadyBound(String),

    /// 验证错误
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Result 类型别名
pub type Result<T> = std::result::Result<T, RegistryError>;

#[cfg(test)]
mod tests {
    // 标准库
    use std::error::Error;

    // 第三方库
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    // 内部导入
    use super::*;

    // 使用 rstest 参数化测试所有错误类型
    #[rstest]
    #[case(RegistryError::NotBound("TestService".to_string()), "Service not bound: TestService")]
    #[case(RegistryError::CircularDependency("A -> B -> A".to_string()), "Circular dependency detected: A -> B -> A")]
    #[case(RegistryError::TypeCast("Failed to cast to Arc<T>".to_string()), "Type cast error: Failed to cast to Arc<T>")]
    #[case(RegistryError::LockError("Failed to acquire lock".to_string()), "Lock error: Failed to acquire lock")]
    #[case(RegistryError::AlreadyBound("TestService".to_string()), "Service already bound: TestService")]
    #[case(RegistryError::ValidationError("Validation failed".to_string()), "Validation error: Validation failed")]
    fn test_error_messages(#[case] error: RegistryError, #[case] expected: &str) {
        let error_msg = format!("{}", error);
        assert_eq!(error_msg, expected);
    }

    #[test]
    fn test_error_debug() {
        let error = RegistryError::NotBound("Test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("NotBound"));
        assert!(debug_str.contains("Test"));
    }

    #[test]
    fn test_result_type_alias() {
        // 测试 Result 类型别名是否正常工作
        let success: Result<i32> = Ok(42);
        assert!(matches!(success, Ok(42)));

        let failure: Result<i32> = Err(RegistryError::NotBound("Test".to_string()));
        assert!(failure.is_err());
    }

    #[test]
    fn test_error_source() {
        // 测试 thiserror 的 source 功能
        let error = RegistryError::NotBound("Test".to_string());
        // thiserror 自动实现了 std::error::Error trait
        assert!(error.source().is_none()); // NotBound 没有 source
    }

    #[test]
    fn test_error_snapshot() {
        // 使用 insta 快照测试错误消息格式
        let error = RegistryError::CircularDependency("A -> B -> C -> A".to_string());
        assert_snapshot!(format!("{}", error), @"Circular dependency detected: A -> B -> C -> A");
    }
}
