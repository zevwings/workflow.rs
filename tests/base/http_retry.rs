//! Base/HTTP/Retry 模块测试
//!
//! 测试HTTP重试机制的核心业务逻辑，包括：
//! - 重试策略判断（哪些错误可重试）
//! - 指数退避算法计算
//! - 错误分类和描述生成
//! - 重试配置验证
//!
//! 注意：我们不测试实际的HTTP请求，只测试重试逻辑本身

use std::time::{Duration, Instant};

use color_eyre::{eyre::eyre, Result};
use rstest::rstest;

use workflow::base::http::retry::{HttpRetry, HttpRetryConfig};

use std::sync::{Arc, Mutex};

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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== 配置测试 ====================

    #[test]
    fn test_retry_config_default() {
        let config = HttpRetryConfig::default();

        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, 1);
        assert_eq!(config.max_delay, 30);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.interactive, true);
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
        assert_eq!(config.interactive, default_config.interactive);
    }

    #[test]
    fn test_retry_config_custom() {
        let config = HttpRetryConfig {
            max_retries: 5,
            initial_delay: 2,
            max_delay: 60,
            backoff_multiplier: 1.5,
            interactive: false,
        };

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay, 2);
        assert_eq!(config.max_delay, 60);
        assert_eq!(config.backoff_multiplier, 1.5);
        assert_eq!(config.interactive, false);
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
            interactive: false, // 非交互模式，避免用户输入
        };

        let result =
            HttpRetry::retry(create_always_success_operation(), &config, "test operation").unwrap();

        assert_eq!(result.retry_count, 0);
        assert_eq!(result.succeeded_on_first_attempt, true);
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
            interactive: false,
        };

        // 第2次尝试成功（第1次失败，第2次成功）
        let result =
            HttpRetry::retry(create_success_after_attempts(2), &config, "test operation").unwrap();

        assert_eq!(result.retry_count, 1); // 重试了1次
        assert_eq!(result.succeeded_on_first_attempt, false);
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
            interactive: false,
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
            interactive: false,
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
            interactive: false,
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
            interactive: false,
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
            interactive: false,
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
            interactive: false,
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
            interactive: false,
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
            interactive: false,
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
            interactive: false,
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
            interactive: false,
        };

        // 测试整数返回类型
        let int_result =
            HttpRetry::retry(|| -> Result<i32> { Ok(42) }, &config, "int test").unwrap();

        assert_eq!(int_result.result, 42);

        // 测试布尔返回类型
        let bool_result =
            HttpRetry::retry(|| -> Result<bool> { Ok(true) }, &config, "bool test").unwrap();

        assert_eq!(bool_result.result, true);

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
            interactive: false,
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
            interactive: false,
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
            interactive: false,
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
}
