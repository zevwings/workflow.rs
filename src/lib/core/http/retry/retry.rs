//! HTTP 重试逻辑

use color_eyre::Result;
use std::time::{Duration, Instant};

use crate::core::http::retry::config::{HttpRetryConfig, RetryResult};
use crate::core::http::retry::error::HttpRetryError;
/// HTTP 重试工具
///
/// 提供 HTTP 请求重试的功能，支持指数退避算法。
/// 专门针对 HTTP 请求的错误类型进行智能判断。
pub struct HttpRetry;

impl HttpRetry {
    /// 判断错误是否可重试
    ///
    /// 检查错误类型，判断是否应该重试。
    /// 可重试的错误包括：
    /// - 网络错误（超时、连接失败、请求中断）
    /// - 5xx 服务器错误（500, 502, 503, 504）
    /// - 429 Too Many Requests（需要特殊处理，使用 Retry-After header）
    ///
    /// 不可重试的错误包括：
    /// - 4xx 客户端错误（400, 401, 403, 404 等）
    /// - 解析错误（JSON 解析失败、文件格式错误）
    /// - 其他非网络错误
    fn is_retryable_error(error: &color_eyre::eyre::Report) -> bool {
        if let Some(reqwest_error) = error.downcast_ref::<reqwest::Error>() {
            if reqwest_error.is_timeout()
                || reqwest_error.is_connect()
                || reqwest_error.is_request()
            {
                return true;
            }

            if let Some(status) = reqwest_error.status() {
                return status.is_server_error() || status.as_u16() == 429;
            }
        }

        if let Some(io_error) = error.downcast_ref::<std::io::Error>() {
            match io_error.kind() {
                std::io::ErrorKind::TimedOut
                | std::io::ErrorKind::ConnectionRefused
                | std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::ConnectionAborted
                | std::io::ErrorKind::NotConnected
                | std::io::ErrorKind::BrokenPipe => return true,
                _ => {}
            }
        }

        false
    }

    /// 从错误中提取可读的错误描述
    ///
    /// 尝试从错误中提取有用的信息，用于日志输出。
    fn get_error_description(error: &color_eyre::eyre::Report) -> String {
        const MAX_ERROR_MSG_LENGTH: usize = 100;
        const ELLIPSIS: &str = "...";

        if let Some(reqwest_error) = error.downcast_ref::<reqwest::Error>() {
            if let Some(status) = reqwest_error.status() {
                return format!("HTTP {}", status.as_u16());
            }
            if reqwest_error.is_timeout() {
                return "Network timeout".to_string();
            }
            if reqwest_error.is_connect() {
                return "Connection failed".to_string();
            }
        }

        if let Some(io_error) = error.downcast_ref::<std::io::Error>() {
            return format!("IO error: {}", io_error.kind());
        }

        let error_msg = error.to_string();
        if error_msg.len() > MAX_ERROR_MSG_LENGTH {
            format!("{}{}", &error_msg[..MAX_ERROR_MSG_LENGTH], ELLIPSIS)
        } else {
            error_msg
        }
    }

    /// 从错误中提取 Retry-After header 的值
    ///
    /// 对于 429 Too Many Requests 错误，尝试从响应中提取 Retry-After header。
    /// Retry-After 可以是秒数（整数）或 HTTP 日期格式。
    ///
    /// 注意：在 blocking 模式下，reqwest::Error 可能不包含响应信息。
    /// 如果需要完整的 Retry-After 支持，建议在 HttpResponse 中检查 header。
    fn extract_retry_after(error: &color_eyre::eyre::Report) -> Option<u64> {
        if let Some(reqwest_error) = error.downcast_ref::<reqwest::Error>() {
            if let Some(status) = reqwest_error.status() {
                if status.as_u16() == 429 {
                    // 在 blocking 模式下，reqwest::Error 可能不包含响应，这里返回 None
                }
            }
        }

        // 检查是否是 HttpClientError::RateLimitExceeded
        // 注意：HttpClientError 可能不包含响应信息，所以这里无法提取 Retry-After
        // 如果需要，可以在 HttpClientError 中添加响应信息

        None
    }

    /// 使用指数退避算法重试 HTTP 操作
    ///
    /// 执行一个可能失败的 HTTP 操作，如果失败且错误可重试，则按照配置的重试策略进行重试。
    /// 使用指数退避算法，每次重试的延迟时间会逐渐增加，直到达到最大延迟。
    ///
    /// # 参数
    ///
    /// * `operation` - 要执行的操作（闭包）
    /// * `config` - 重试配置
    /// * `operation_name` - 操作名称（用于日志输出）
    ///
    /// # 类型参数
    ///
    /// * `F` - 操作闭包类型，必须返回 `Result<T>`
    /// * `T` - 操作返回值的类型
    ///
    /// # 返回
    ///
    /// 返回操作的结果和重试信息。如果所有重试都失败，返回最后一次的错误。
    ///
    /// # 错误处理
    ///
    /// - 如果错误不可重试（如 4xx 客户端错误），立即返回错误，不进行重试
    /// - 如果所有重试都失败，返回最后一次的错误，并附加重试信息
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::retry::{HttpRetry, HttpRetryConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = HttpRetryConfig::new();
    /// let result = HttpRetry::retry(
    ///     || {
    ///         // 执行可能失败的 HTTP 操作
    ///         Ok(42)
    ///     },
    ///     &config,
    ///     "获取数据",
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn retry<F, T>(
        operation: F,
        config: &HttpRetryConfig,
        operation_name: &str,
    ) -> Result<RetryResult<T>>
    where
        F: Fn() -> Result<T>,
    {
        let span = tracing::span!(
            tracing::Level::DEBUG,
            "http.retry",
            module = "http",
            operation = %operation_name,
            max_retries = config.max_retries,
        );
        let _guard = span.enter();
        let start = Instant::now();

        let mut delay = config.initial_delay;
        let mut last_error = None;

        for attempt in 0..=config.max_retries {
            let attempt_span = tracing::span!(
                tracing::Level::DEBUG,
                "http.retry.attempt",
                module = "http",
                operation = %operation_name,
                attempt = attempt + 1,
                max_retries = config.max_retries + 1,
            );
            let _attempt_guard = attempt_span.enter();

            match operation() {
                Ok(result) => {
                    let duration = start.elapsed();
                    tracing::info!(
                        module = "http",
                        operation = %operation_name,
                        retry_count = attempt,
                        succeeded_on_first_attempt = attempt == 0,
                        http.retry.duration_ms = duration.as_millis(),
                        "Operation succeeded"
                    );
                    return Ok(RetryResult {
                        result,
                        retry_count: attempt,
                        succeeded_on_first_attempt: attempt == 0,
                    });
                }
                Err(e) => {
                    let error = e;
                    last_error = Some(error);

                    // 检查是否可重试
                    if let Some(ref err) = last_error {
                        if !Self::is_retryable_error(err) {
                            // 错误不可重试，立即返回
                            tracing::warn!(
                                module = "http",
                                operation = %operation_name,
                                attempt = attempt + 1,
                                error = %err,
                                "Error is not retryable"
                            );
                            return Err(last_error.ok_or_else(
                                || -> color_eyre::eyre::Report {
                                    HttpRetryError::NoErrorAvailable.into()
                                },
                            )?);
                        }
                    }

                    // 如果还有重试机会
                    if attempt < config.max_retries {
                        if let Some(ref err) = last_error {
                            let error_desc = HttpRetry::get_error_description(err);

                            // 检查是否是 429 错误，并尝试提取 Retry-After header
                            let retry_after = Self::extract_retry_after(err);
                            let actual_delay = retry_after.unwrap_or(delay);

                            tracing::warn!(
                                module = "http",
                                operation = %operation_name,
                                attempt = attempt + 1,
                                max_retries = config.max_retries + 1,
                                error = %error_desc,
                                delay_seconds = actual_delay,
                                retry_after = retry_after.map(|d| d.to_string()),
                                "Operation failed, will retry"
                            );

                            // 等待后重试（使用 Retry-After 或指数退避延迟）
                            std::thread::sleep(Duration::from_secs(actual_delay));

                            // 如果使用了 Retry-After，下次仍然使用指数退避
                            // 否则继续指数退避
                            if retry_after.is_none() {
                                delay = ((delay as f64 * config.backoff_multiplier) as u64)
                                    .min(config.max_delay);
                            } else {
                                // 使用 Retry-After 后，重置延迟为初始延迟的倍数
                                delay = ((config.initial_delay as f64 * config.backoff_multiplier)
                                    as u64)
                                    .min(config.max_delay);
                            }
                        }
                    } else {
                        // 所有重试都失败了
                        if let Some(ref err) = last_error {
                            let error_desc = HttpRetry::get_error_description(err);
                            tracing::error!(
                                module = "http",
                                operation = %operation_name,
                                retry_count = config.max_retries,
                                error = %error_desc,
                                "All retries exhausted"
                            );
                        }
                    }
                }
            }
        }

        // 所有重试都失败，返回最后一次的错误，并添加上下文信息
        let duration = start.elapsed();
        let final_error: color_eyre::eyre::Report =
            last_error.ok_or_else(|| -> color_eyre::eyre::Report {
                HttpRetryError::AllRetriesFailedNoError.into()
            })?;
        tracing::error!(
            module = "http",
            operation = %operation_name,
            retries = config.max_retries,
            http.retry.duration_ms = duration.as_millis(),
            "Operation failed after all retries"
        );
        Err(HttpRetryError::OperationFailedAfterRetries {
            operation: operation_name.to_string(),
            retries: config.max_retries,
            source: final_error,
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::{eyre::eyre, Result};
    use rstest::rstest;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    /// 创建在指定次数后成功的操作（使用局部计数器避免并发问题）
    fn create_success_after_attempts(success_after: usize) -> impl Fn() -> Result<String> {
        let counter = Arc::new(Mutex::new(0usize));
        move || {
            let mut count = counter.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count); // 释放锁

            if current >= success_after {
                Ok("success".to_string())
            } else {
                Err(eyre!(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "temporary failure"
                )))
            }
        }
    }

    /// 创建总是失败的操作（使用可重试的错误）
    fn create_always_fail_operation() -> impl Fn() -> Result<String> {
        || {
            // 创建一个模拟的网络超时错误，这是可重试的
            let io_error = std::io::Error::new(std::io::ErrorKind::TimedOut, "connection timeout");
            Err(eyre!(io_error))
        }
    }

    /// 创建总是成功的操作
    fn create_always_success_operation() -> impl Fn() -> Result<String> {
        || Ok("immediate success".to_string())
    }

    /// 重置测试计数器（现在不需要了，因为每个操作都有自己的计数器）
    fn reset_counters() {
        // 不再需要重置全局计数器，因为每个 create_success_after_attempts 都有自己的计数器
    }

    // ==================== 配置测试 ====================

    #[test]
    fn test_retry_config_default() {
        let config = HttpRetryConfig::default();

        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, 1);
        assert_eq!(config.max_delay, 30);
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_retry_config_new() {
        let config = HttpRetryConfig::new();

        // new() 应该等同于 default()
        let default_config = HttpRetryConfig::default();
        assert_eq!(config.max_retries, default_config.max_retries);
        assert_eq!(config.initial_delay, default_config.initial_delay);
        assert_eq!(config.max_delay, default_config.max_delay);
        assert_eq!(config.backoff_multiplier, default_config.backoff_multiplier);
    }

    #[test]
    fn test_retry_config_custom() {
        let config = HttpRetryConfig {
            max_retries: 5,
            initial_delay: 2,
            max_delay: 60,
            backoff_multiplier: 1.5,
        };

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay, 2);
        assert_eq!(config.max_delay, 60);
        assert_eq!(config.backoff_multiplier, 1.5);
    }

    // ==================== 基础重试逻辑测试 ====================

    #[test]
    fn test_immediate_success() {
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 3,
            initial_delay: 1,
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        let result =
            HttpRetry::retry(create_always_success_operation(), &config, "test operation").unwrap();

        assert_eq!(result.retry_count, 0);
        assert!(result.succeeded_on_first_attempt);
        assert_eq!(result.result, "immediate success");
    }

    #[test]
    fn test_success_after_retries() {
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 3,
            initial_delay: 0, // 设为0以加快测试
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        // 第2次尝试成功（第1次失败，第2次成功）
        let result =
            HttpRetry::retry(create_success_after_attempts(2), &config, "test operation").unwrap();

        assert_eq!(result.retry_count, 1); // 重试了1次
        assert!(!result.succeeded_on_first_attempt);
        assert_eq!(result.result, "success");
    }

    #[test]
    fn test_all_retries_exhausted() {
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 2,
            initial_delay: 0,
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        let result = HttpRetry::retry(create_always_fail_operation(), &config, "test operation");

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("test operation failed after 2 retries"));
    }

    // ==================== 指数退避算法测试 ====================

    #[test]
    #[ignore] // 需要等待约3秒，影响测试速度
    fn test_backoff_timing() {
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 2,
            initial_delay: 1, // 1秒初始延迟
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        let start_time = Instant::now();

        // 这个操作会失败3次（超过max_retries），测试总时间
        let _result = HttpRetry::retry(create_always_fail_operation(), &config, "timing test");

        let duration = start_time.elapsed();

        // 预期时间：第1次重试等1秒，第2次重试等2秒，总共约3秒
        // 允许一些误差
        assert!(duration >= Duration::from_millis(2800)); // 至少2.8秒
        assert!(duration <= Duration::from_millis(4000)); // 最多4秒
    }

    #[rstest]
    #[case(1, 2.0, 30, vec![1, 2, 4, 8, 16, 30, 30])] // 标准指数退避
    #[case(2, 1.5, 10, vec![2, 3, 4, 6, 9, 10, 10])] // 不同参数
    #[case(5, 3.0, 20, vec![5, 15, 20, 20, 20])] // 快速达到最大值
    #[ignore] // 需要等待很长时间（case1: 91秒, case2: 44秒, case3: 80秒），影响测试速度
    fn test_backoff_calculation(
        #[case] initial_delay: u64,
        #[case] multiplier: f64,
        #[case] max_delay: u64,
        #[case] expected_delays: Vec<u64>,
    ) {
        // 这个测试验证退避算法的计算逻辑
        // 由于我们无法直接访问内部的延迟计算，我们通过测试多次失败的时间来验证

        let config = HttpRetryConfig {
            max_retries: expected_delays.len() as u32,
            initial_delay,
            max_delay,
            backoff_multiplier: multiplier,
        };

        let start_time = Instant::now();
        let _result = HttpRetry::retry(create_always_fail_operation(), &config, "backoff test");
        let duration = start_time.elapsed();

        // 计算预期总时间
        let expected_total_seconds: u64 = expected_delays.iter().sum();
        let expected_duration = Duration::from_secs(expected_total_seconds);

        // 允许±500ms的误差
        let min_expected = expected_duration.saturating_sub(Duration::from_millis(500));
        let max_expected = expected_duration + Duration::from_millis(500);

        assert!(
            duration >= min_expected && duration <= max_expected,
            "Duration {:?} not in expected range [{:?}, {:?}] for delays {:?}",
            duration,
            min_expected,
            max_expected,
            expected_delays
        );
    }

    // ==================== 错误处理测试 ====================

    #[test]
    fn test_retry_result_structure() {
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 2,
            initial_delay: 0,
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        // 测试立即成功的情况
        let success_result =
            HttpRetry::retry(create_always_success_operation(), &config, "success test").unwrap();

        assert_eq!(success_result.retry_count, 0);
        assert!(success_result.succeeded_on_first_attempt);
        assert_eq!(success_result.result, "immediate success");

        // 重置计数器
        reset_counters();

        // 测试重试后成功的情况
        let retry_success_result = HttpRetry::retry(
            create_success_after_attempts(2),
            &config,
            "retry success test",
        )
        .unwrap();

        assert_eq!(retry_success_result.retry_count, 1);
        assert!(!retry_success_result.succeeded_on_first_attempt);
        assert_eq!(retry_success_result.result, "success");
    }

    #[test]
    fn test_operation_name_in_error() {
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 1,
            initial_delay: 0,
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        let operation_name = "custom operation name";
        let result = HttpRetry::retry(create_always_fail_operation(), &config, operation_name);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains(operation_name));
        assert!(error_msg.contains("failed after 1 retries"));
    }

    // ==================== 边界条件测试 ====================

    #[test]
    fn test_zero_max_retries() {
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 0, // 不重试
            initial_delay: 1,
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        // 成功操作应该立即返回
        let success_result = HttpRetry::retry(
            create_always_success_operation(),
            &config,
            "no retry success",
        )
        .unwrap();

        assert_eq!(success_result.retry_count, 0);
        assert!(success_result.succeeded_on_first_attempt);

        // 失败操作应该立即失败，不重试
        let start_time = Instant::now();
        let fail_result =
            HttpRetry::retry(create_always_fail_operation(), &config, "no retry fail");
        let duration = start_time.elapsed();

        assert!(fail_result.is_err());
        // 应该很快失败，没有延迟
        assert!(duration < Duration::from_millis(100));
    }

    #[test]
    fn test_large_max_retries() {
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 100, // 很大的重试次数
            initial_delay: 0, // 无延迟以加快测试
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        // 第5次尝试成功
        let result = HttpRetry::retry(
            create_success_after_attempts(5),
            &config,
            "large retry test",
        )
        .unwrap();

        assert_eq!(result.retry_count, 4); // 重试了4次
        assert!(!result.succeeded_on_first_attempt);
        assert_eq!(result.result, "success");
    }

    #[test]
    fn test_zero_initial_delay() {
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 2,
            initial_delay: 0, // 零延迟
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        let start_time = Instant::now();
        let _result = HttpRetry::retry(create_always_fail_operation(), &config, "zero delay test");
        let duration = start_time.elapsed();

        // 零延迟应该很快完成
        assert!(duration < Duration::from_millis(100));
    }

    #[test]
    #[ignore] // 需要等待约20秒，影响测试速度
    fn test_max_delay_limit() {
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 10,
            initial_delay: 1,
            max_delay: 2,             // 很小的最大延迟
            backoff_multiplier: 10.0, // 很大的倍数
        };

        let start_time = Instant::now();
        let _result = HttpRetry::retry(create_always_fail_operation(), &config, "max delay test");
        let duration = start_time.elapsed();

        // 即使倍数很大，也应该被max_delay限制
        // 10次重试，每次最多2秒，总共最多20秒，加上一些误差
        assert!(duration <= Duration::from_secs(25));
    }

    // ==================== 类型和泛型测试 ====================

    #[test]
    fn test_different_return_types() {
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 1,
            initial_delay: 0,
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        // 测试整数返回类型
        let int_result =
            HttpRetry::retry(|| -> Result<i32> { Ok(42) }, &config, "int test").unwrap();

        assert_eq!(int_result.result, 42);

        // 测试布尔返回类型
        let bool_result =
            HttpRetry::retry(|| -> Result<bool> { Ok(true) }, &config, "bool test").unwrap();

        assert!(bool_result.result);

        // 测试自定义结构体
        #[derive(Debug, PartialEq)]
        struct CustomData {
            id: u32,
            name: String,
        }

        let custom_data = CustomData {
            id: 123,
            name: "test".to_string(),
        };

        let custom_result = HttpRetry::retry(
            || -> Result<CustomData> {
                Ok(CustomData {
                    id: 123,
                    name: "test".to_string(),
                })
            },
            &config,
            "custom test",
        )
        .unwrap();

        assert_eq!(custom_result.result, custom_data);
    }

    #[test]
    fn test_different_error_types() {
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 1,
            initial_delay: 0,
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        // 测试字符串错误（转换为color_eyre::Result）
        let string_error_result = HttpRetry::retry(
            || -> Result<String> { Err(color_eyre::eyre::eyre!("string error")) },
            &config,
            "string error test",
        );

        assert!(string_error_result.is_err());

        // 测试自定义错误类型
        #[derive(Debug)]
        struct CustomError {
            code: i32,
            message: String,
        }

        impl std::fmt::Display for CustomError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "CustomError {}: {}", self.code, self.message)
            }
        }

        impl std::error::Error for CustomError {}

        let custom_error_result = HttpRetry::retry(
            || -> Result<String> {
                Err(CustomError {
                    code: 404,
                    message: "Not found".to_string(),
                }
                .into())
            },
            &config,
            "custom error test",
        );

        assert!(custom_error_result.is_err());
    }

    // ==================== 性能和稳定性测试 ====================

    #[test]
    fn test_rapid_successive_calls() {
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 1,
            initial_delay: 0,
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        // 快速连续调用多次
        for i in 0..10 {
            let result = HttpRetry::retry(
                || -> Result<usize> { Ok(i) },
                &config,
                &format!("rapid call {}", i),
            )
            .unwrap();

            assert_eq!(result.result, i);
            assert_eq!(result.retry_count, 0);
            assert!(result.succeeded_on_first_attempt);
        }
    }

    #[test]
    fn test_consistent_behavior() {
        // 测试相同配置下的行为一致性
        let config = HttpRetryConfig {
            max_retries: 2,
            initial_delay: 0,
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        // 多次运行相同的测试，验证行为一致
        for _ in 0..5 {
            reset_counters();

            let result = HttpRetry::retry(
                create_success_after_attempts(2),
                &config,
                "consistency test",
            )
            .unwrap();

            assert_eq!(result.retry_count, 1);
            assert!(!result.succeeded_on_first_attempt);
            assert_eq!(result.result, "success");
        }
    }

    // ==================== 可重试错误判断测试（通过 retry 方法间接测试） ====================

    #[test]
    fn test_retryable_timeout_error() {
        // 测试超时错误是否可重试
        // 通过创建超时错误并验证会进行重试
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 2,
            initial_delay: 0,
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        // 创建超时错误（可重试）
        let timeout_error = || -> Result<String> {
            Err(color_eyre::eyre::eyre!(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "connection timeout"
            )))
        };

        let result = HttpRetry::retry(timeout_error, &config, "timeout test");
        // 应该重试多次后失败
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("timeout test"));
    }

    #[test]
    fn test_retryable_connection_error() {
        // 测试连接错误是否可重试
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 1,
            initial_delay: 0,
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        // 创建连接错误（可重试）
        let connection_error = || -> Result<String> {
            Err(color_eyre::eyre::eyre!(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "connection refused"
            )))
        };

        let result = HttpRetry::retry(connection_error, &config, "connection test");
        // 应该重试后失败
        assert!(result.is_err());
    }

    #[test]
    fn test_non_retryable_4xx_error() {
        // 测试 4xx 错误是否不可重试（通过模拟 reqwest 错误）
        // 注意：由于 is_retryable_error 是私有方法，我们通过 retry 方法的行为来测试
        // 4xx 错误应该立即返回，不进行重试
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 3,
            initial_delay: 0,
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        // 创建 4xx 错误（不可重试）
        // 由于无法直接创建 reqwest::Error，我们使用其他不可重试的错误类型
        // 例如：解析错误
        let parse_error =
            || -> Result<String> { Err(color_eyre::eyre::eyre!("Invalid JSON: unexpected token")) };

        let start_time = Instant::now();
        let result = HttpRetry::retry(parse_error, &config, "4xx test");
        let duration = start_time.elapsed();

        // 应该立即失败，不进行重试（没有延迟）
        assert!(result.is_err());
        assert!(
            duration < Duration::from_millis(100),
            "Should fail immediately without retry"
        );
    }

    // ==================== 错误描述提取测试 ====================

    #[test]
    fn test_error_description_long_message() {
        // 测试长错误消息的截断（超过 100 字符）
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 0, // 不重试，立即失败
            initial_delay: 0,
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        // 创建长错误消息，使用可重试的错误（TimedOut）以确保错误被包装在 HttpRetryError 中
        let long_error_msg = "a".repeat(150);
        let long_error = || -> Result<String> {
            let io_error =
                std::io::Error::new(std::io::ErrorKind::TimedOut, long_error_msg.clone());
            Err(color_eyre::eyre::eyre!(io_error))
        };

        let result = HttpRetry::retry(long_error, &config, "long error test");
        assert!(result.is_err());

        // 验证错误消息被截断（通过错误描述）
        // 注意：实际的截断逻辑在 get_error_description 中
        // 但由于它是私有方法，我们通过错误消息来验证
        let error_msg = result.unwrap_err().to_string();
        // 错误消息应该包含操作名称（因为错误被包装在 HttpRetryError 中）
        assert!(error_msg.contains("long error test"));
    }

    #[test]
    fn test_error_description_short_message() {
        // 测试短错误消息（不超过 100 字符）
        reset_counters();
        let config = HttpRetryConfig {
            max_retries: 0,
            initial_delay: 0,
            max_delay: 30,
            backoff_multiplier: 2.0,
        };

        // 使用可重试的错误（TimedOut）以确保错误被包装在 HttpRetryError 中
        let short_error_msg = "Short error message";
        let short_error = || -> Result<String> {
            let io_error = std::io::Error::new(std::io::ErrorKind::TimedOut, short_error_msg);
            Err(color_eyre::eyre::eyre!(io_error))
        };

        let result = HttpRetry::retry(short_error, &config, "short error test");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        // 错误消息应该包含操作名称（因为错误被包装在 HttpRetryError 中）
        assert!(error_msg.contains("short error test"));
    }
}
