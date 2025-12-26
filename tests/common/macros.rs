//! 测试辅助宏
//!
//! 提供测试中常用的宏，简化错误处理和断言操作。
//!
//! ## 使用示例
//!
//! ```rust
//! use tests::common::macros::*;
//!
//! #[test]
//! fn test_something() -> color_eyre::Result<()> {
//!     let value = test_result!(some_function())?;
//!     test_assert!(value > 0, "Value should be positive");
//!     Ok(())
//! }
//! ```

/// 测试断言宏
///
/// 在测试中用于断言，如果失败会提供详细的错误信息。
/// 这是 `expect()` 的包装，提供更清晰的测试错误消息。
///
/// # 参数
///
/// * `$expr` - 要断言的表达式（应该返回 `Result` 或 `Option`）
/// * `$msg` - 失败时显示的错误消息
///
/// # 示例
///
/// ```rust
/// use tests::common::macros::test_assert;
///
/// #[test]
/// fn test_example() {
///     let result: Result<i32, &str> = Ok(42);
///     let value = test_assert!(result, "Expected successful result");
///     assert_eq!(value, 42);
/// }
/// ```
#[macro_export]
macro_rules! test_assert {
    ($expr:expr, $msg:expr) => {
        $expr.expect($msg)
    };
}

/// 测试结果转换宏
///
/// 将任意错误类型转换为 `color_eyre::eyre::Report`，便于在测试中使用 `?` 操作符。
///
/// # 参数
///
/// * `$expr` - 返回 `Result` 的表达式
///
/// # 返回
///
/// 返回 `color_eyre::Result<T>`，其中 `T` 是原 `Result` 的成功类型。
///
/// # 示例
///
/// ```rust
/// use tests::common::macros::test_result;
/// use color_eyre::Result;
///
/// #[test]
/// fn test_example() -> Result<()> {
///     let value: Result<i32, String> = Ok(42);
///     let result = test_result!(value)?;
///     assert_eq!(result, 42);
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! test_result {
    ($expr:expr) => {
        $expr.map_err(|e| color_eyre::eyre::eyre!("Test failed: {}", e))
    };
}

#[cfg(test)]
mod tests {
    use color_eyre::Result;

    #[test]
    fn test_test_assert_with_ok_result() {
        let result: Result<i32, &str> = Ok(42);
        let value = test_assert!(result, "Expected successful result");
        assert_eq!(value, 42);
    }

    #[test]
    #[should_panic(expected = "Expected successful result")]
    fn test_test_assert_with_err_result() {
        let result: Result<i32, &str> = Err("error");
        let _value = test_assert!(result, "Expected successful result");
    }

    #[test]
    fn test_test_assert_with_some_option() {
        let option: Option<i32> = Some(42);
        let value = test_assert!(option, "Expected Some value");
        assert_eq!(value, 42);
    }

    #[test]
    #[should_panic(expected = "Expected Some value")]
    fn test_test_assert_with_none_option() {
        let option: Option<i32> = None;
        let _value = test_assert!(option, "Expected Some value");
    }

    #[test]
    fn test_test_result_with_ok() -> Result<()> {
        let result: Result<i32, String> = Ok(42);
        let value = test_result!(result)?;
        assert_eq!(value, 42);
        Ok(())
    }

    #[test]
    fn test_test_result_with_err() {
        let result: Result<i32, String> = Err("test error".to_string());
        let converted = test_result!(result);
        assert!(converted.is_err());
        assert!(converted
            .unwrap_err()
            .to_string()
            .contains("Test failed: test error"));
    }
}

