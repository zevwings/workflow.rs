#![allow(clippy::test_attr_in_doctest)]

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
        assert!(converted.unwrap_err().to_string().contains("Test failed: test error"));
    }
}

/// 模式匹配断言宏
///
/// 验证值是否匹配指定的模式，失败时提供详细的错误信息。
///
/// # 参数
///
/// * `$value` - 要匹配的值
/// * `$pattern` - 匹配模式（Rust 模式语法）
/// * `$msg` - 可选的错误消息
///
/// # 示例
///
/// ```rust
/// use tests::common::macros::assert_matches;
///
/// #[test]
/// fn test_pattern_matching() {
///     let result: Result<i32, &str> = Ok(42);
///     assert_matches!(result, Ok(x) if x > 0);
///     assert_matches!(result, Ok(42));
/// }
/// ```
#[macro_export]
macro_rules! assert_matches {
    ($value:expr, $pattern:pat) => {
        match $value {
            $pattern => {}
            ref v => panic!(
                "assertion failed: `{:?}` does not match pattern `{}`",
                v,
                stringify!($pattern)
            ),
        }
    };
    ($value:expr, $pattern:pat if $guard:expr) => {
        match $value {
            $pattern if $guard => {}
            ref v => panic!(
                "assertion failed: `{:?}` does not match pattern `{}` with guard",
                v,
                stringify!($pattern)
            ),
        }
    };
    ($value:expr, $pattern:pat, $msg:expr) => {
        match $value {
            $pattern => {}
            ref v => panic!(
                "{}: `{:?}` does not match pattern `{}`",
                $msg,
                v,
                stringify!($pattern)
            ),
        }
    };
}

/// 带超时的测试宏
///
/// 为测试添加超时机制，防止测试无限期运行。
///
/// # 参数
///
/// * `$timeout` - 超时时间（`Duration` 或秒数）
/// * `$test` - 测试代码块
///
/// # 示例
///
/// ```rust
/// use tests::common::macros::test_with_timeout;
/// use std::time::Duration;
///
/// #[test]
/// fn test_with_timeout_example() {
///     test_with_timeout!(Duration::from_secs(5), {
///         // 测试代码，如果超过5秒会失败
///         std::thread::sleep(Duration::from_secs(2));
///     });
/// }
/// ```
#[macro_export]
macro_rules! test_with_timeout {
    ($timeout:expr, $test:block) => {{
        use std::sync::{Arc, Mutex};
        use std::thread;
        use std::time::{Duration, Instant};

        let timeout = match $timeout {
            d if d.as_secs() > 0 || d.as_nanos() > 0 => d,
            _ => Duration::from_secs(5), // 默认5秒
        };

        let result = Arc::new(Mutex::new(None));
        let result_clone = result.clone();

        let handle = thread::spawn(move || {
            let start = Instant::now();
            let test_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $test));
            let duration = start.elapsed();
            *result_clone.lock().unwrap() = Some((test_result, duration));
        });

        // 等待测试完成或超时
        let start = Instant::now();
        while start.elapsed() < timeout {
            if let Ok(guard) = result.lock() {
                if guard.is_some() {
                    break;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }

        if handle.is_finished() {
            let _ = handle.join();
        } else {
            panic!("Test timed out after {:?}", timeout);
        }

        let (test_result, duration) = result.lock().unwrap().take().unwrap();
        match test_result {
            Ok(_) => {
                if duration > timeout {
                    panic!("Test exceeded timeout: {:?} > {:?}", duration, timeout);
                }
            }
            Err(e) => std::panic::resume_unwind(e),
        }
    }};
}

/// 近似相等断言宏（用于浮点数比较）
///
/// 比较两个浮点数是否近似相等，避免浮点数精度问题。
///
/// # 参数
///
/// * `$left` - 左侧值
/// * `$right` - 右侧值
/// * `$epsilon` - 可选的误差范围（默认 1e-6）
///
/// # 示例
///
/// ```rust
/// use tests::common::macros::assert_approx_eq;
///
/// #[test]
/// fn test_float_comparison() {
///     assert_approx_eq!(0.1 + 0.2, 0.3);
///     assert_approx_eq!(1.0, 1.0000001, 1e-5);
/// }
/// ```
#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr) => {
        assert_approx_eq!($left, $right, 1e-6);
    };
    ($left:expr, $right:expr, $epsilon:expr) => {
        {
            let left: f64 = $left as f64;
            let right: f64 = $right as f64;
            let epsilon: f64 = $epsilon as f64;
            let diff = (left - right).abs();
            if diff > epsilon {
                panic!("assertion failed: `{:?}` is not approximately equal to `{:?}` (difference: {:?}, epsilon: {:?})",
                    left, right, diff, epsilon);
            }
        }
    };
}

/// 断言错误消息包含指定文本
///
/// 验证错误消息是否包含预期的文本，用于测试错误处理。
///
/// # 参数
///
/// * `$result` - `Result` 或 `Option` 值
/// * `$pattern` - 预期的文本模式（字符串）
///
/// # 示例
///
/// ```rust
/// use tests::common::macros::assert_error_contains;
///
/// #[test]
/// fn test_error_message() {
///     let result: Result<(), &str> = Err("File not found");
///     assert_error_contains!(result, "not found");
/// }
/// ```
#[macro_export]
macro_rules! assert_error_contains {
    ($result:expr, $pattern:expr) => {{
        let result = $result;
        let pattern = $pattern;
        match result {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(e) => {
                let error_msg = format!("{}", e);
                if !error_msg.contains(pattern) {
                    panic!(
                        "Expected error message to contain '{}', but got: {}",
                        pattern, error_msg
                    );
                }
            }
        }
    }};
}

#[cfg(test)]
mod extended_tests {
    use color_eyre::Result;

    #[test]
    fn test_assert_matches_with_ok() {
        let result: Result<i32, &str> = Ok(42);
        assert_matches!(result, Ok(_));
    }

    #[test]
    fn test_assert_matches_with_specific_value() {
        let result: Result<i32, &str> = Ok(42);
        assert_matches!(result, Ok(42));
    }

    #[test]
    fn test_assert_matches_with_guard() {
        let result: Result<i32, &str> = Ok(42);
        assert_matches!(result, Ok(x) if x > 0);
    }

    #[test]
    #[should_panic(expected = "does not match pattern")]
    fn test_assert_matches_fails() {
        let result: Result<i32, &str> = Err("error");
        assert_matches!(result, Ok(_));
    }

    #[test]
    fn test_assert_matches_with_message() {
        let result: Result<i32, &str> = Ok(42);
        assert_matches!(result, Ok(_), "Should be Ok");
    }

    #[test]
    fn test_assert_approx_eq_basic() {
        assert_approx_eq!(0.1 + 0.2, 0.3);
    }

    #[test]
    fn test_assert_approx_eq_with_epsilon() {
        assert_approx_eq!(1.0, 1.0000001, 1e-5);
    }

    #[test]
    #[should_panic(expected = "is not approximately equal")]
    fn test_assert_approx_eq_fails() {
        assert_approx_eq!(1.0, 2.0);
    }

    #[test]
    fn test_assert_error_contains() {
        let result: Result<(), &str> = Err("File not found");
        assert_error_contains!(result, "not found");
    }

    #[test]
    fn test_assert_error_contains_with_string_error() {
        let result: Result<(), String> = Err("Network error occurred".to_string());
        assert_error_contains!(result, "Network");
    }

    #[test]
    #[should_panic(expected = "Expected error")]
    fn test_assert_error_contains_with_ok() {
        let result: Result<(), &str> = Ok(());
        assert_error_contains!(result, "error");
    }

    #[test]
    fn test_test_with_timeout_quick() {
        test_with_timeout!(std::time::Duration::from_secs(5), {
            std::thread::sleep(std::time::Duration::from_millis(100));
        });
    }

    #[test]
    #[should_panic(expected = "timed out")]
    fn test_test_with_timeout_exceeds() {
        test_with_timeout!(std::time::Duration::from_millis(100), {
            std::thread::sleep(std::time::Duration::from_secs(1));
        });
    }
}
