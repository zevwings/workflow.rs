//! HTTP 重试机制测试
//!
//! 测试 HTTP 重试机制的各种场景，包括：
//! - 重试配置测试
//! - 重试逻辑测试
//! - 错误判断测试
//! - 指数退避算法测试

use color_eyre::Result;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use workflow::base::http::retry::{HttpRetry, HttpRetryConfig, RetryResult};

// ==================== 重试配置测试 ====================

#[test]
fn test_retry_config_default() {
    let config = HttpRetryConfig::default();
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_delay, 1);
    assert_eq!(config.max_delay, 30);
    assert_eq!(config.backoff_multiplier, 2.0);
    assert!(config.interactive);
}

#[test]
fn test_retry_config_new() {
    let config = HttpRetryConfig::new();
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_delay, 1);
    assert_eq!(config.max_delay, 30);
    assert_eq!(config.backoff_multiplier, 2.0);
    assert!(config.interactive);
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
    assert!(!config.interactive);
}

// ==================== 重试逻辑测试 ====================

#[test]
fn test_retry_success_on_first_attempt() -> Result<()> {
    let config = HttpRetryConfig {
        max_retries: 3,
        initial_delay: 0, // 使用 0 延迟以加快测试
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false, // 非交互模式
    };

    let result = HttpRetry::retry(
        || Ok::<i32, color_eyre::eyre::Report>(42),
        &config,
        "test operation",
    )?;

    assert_eq!(result.result, 42);
    assert_eq!(result.retry_count, 0);
    assert!(result.succeeded_on_first_attempt);
    Ok(())
}

#[test]
fn test_retry_success_after_retries() -> Result<()> {
    let config = HttpRetryConfig {
        max_retries: 3,
        initial_delay: 0, // 使用 0 延迟以加快测试
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let attempt_count = AtomicU32::new(0);
    let result = HttpRetry::retry(
        || {
            let count = attempt_count.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err(color_eyre::eyre::eyre!("Simulated error"))
            } else {
                Ok::<i32, color_eyre::eyre::Report>(42)
            }
        },
        &config,
        "test operation",
    )?;

    assert_eq!(result.result, 42);
    assert_eq!(result.retry_count, 2);
    assert!(!result.succeeded_on_first_attempt);
    assert_eq!(attempt_count.load(Ordering::SeqCst), 3); // 2 次失败 + 1 次成功
    Ok(())
}

#[test]
fn test_retry_all_attempts_fail() {
    let config = HttpRetryConfig {
        max_retries: 2,
        initial_delay: 0, // 使用 0 延迟以加快测试
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let result = HttpRetry::retry(
        || Err::<i32, color_eyre::eyre::Report>(color_eyre::eyre::eyre!("Always fails")),
        &config,
        "test operation",
    );

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("test operation"));
    assert!(error_msg.contains("retries"));
}

#[test]
fn test_retry_non_retryable_error() {
    // 4xx 客户端错误不应该重试
    let config = HttpRetryConfig {
        max_retries: 3,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    // 模拟 400 Bad Request 错误
    let result = HttpRetry::retry(
        || {
            Err::<i32, color_eyre::eyre::Report>(
                color_eyre::eyre::eyre!("{}", reqwest::Error::from(
                    reqwest::StatusCode::BAD_REQUEST
                )),
            )
        },
        &config,
        "test operation",
    );

    assert!(result.is_err());
    // 应该立即返回，不进行重试
}

#[test]
fn test_retry_max_retries_zero() -> Result<()> {
    let config = HttpRetryConfig {
        max_retries: 0,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    // 第一次成功
    let result = HttpRetry::retry(
        || Ok::<i32, color_eyre::eyre::Report>(42),
        &config,
        "test operation",
    )?;

    assert_eq!(result.result, 42);
    assert_eq!(result.retry_count, 0);

    // 第一次失败，不应该重试
    let result = HttpRetry::retry(
        || Err::<i32, color_eyre::eyre::Report>(color_eyre::eyre::eyre!("Error")),
        &config,
        "test operation",
    );

    assert!(result.is_err());
    Ok(())
}

// ==================== 错误判断测试 ====================

#[test]
fn test_retry_network_timeout_error() {
    // 网络超时错误应该可重试
    let config = HttpRetryConfig {
        max_retries: 1,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let attempt_count = AtomicU32::new(0);
    let result = HttpRetry::retry(
        || {
            let count = attempt_count.fetch_add(1, Ordering::SeqCst);
            if count == 0 {
                // 模拟超时错误
                let timeout_error = reqwest::Error::from(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Connection timeout",
                ));
                Err::<i32, color_eyre::eyre::Report>(color_eyre::eyre::eyre!("{}", timeout_error))
            } else {
                Ok::<i32, color_eyre::eyre::Report>(42)
            }
        },
        &config,
        "test operation",
    );

    // 应该重试并成功
    assert!(result.is_ok() || attempt_count.load(Ordering::SeqCst) > 1);
}

#[test]
fn test_retry_5xx_server_error() {
    // 5xx 服务器错误应该可重试
    let config = HttpRetryConfig {
        max_retries: 1,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let attempt_count = AtomicU32::new(0);
    let result = HttpRetry::retry(
        || {
            let count = attempt_count.fetch_add(1, Ordering::SeqCst);
            if count == 0 {
                // 模拟 500 Internal Server Error
                let error = reqwest::Error::from(reqwest::StatusCode::INTERNAL_SERVER_ERROR);
                Err::<i32, color_eyre::eyre::Report>(color_eyre::eyre::eyre!("{}", error))
            } else {
                Ok::<i32, color_eyre::eyre::Report>(42)
            }
        },
        &config,
        "test operation",
    );

    // 应该重试并成功
    assert!(result.is_ok() || attempt_count.load(Ordering::SeqCst) > 1);
}

#[test]
fn test_retry_429_too_many_requests() {
    // 429 Too Many Requests 应该可重试
    let config = HttpRetryConfig {
        max_retries: 1,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let attempt_count = AtomicU32::new(0);
    let result = HttpRetry::retry(
        || {
            let count = attempt_count.fetch_add(1, Ordering::SeqCst);
            if count == 0 {
                // 模拟 429 Too Many Requests
                let error = reqwest::Error::from(reqwest::StatusCode::TOO_MANY_REQUESTS);
                Err::<i32, color_eyre::eyre::Report>(color_eyre::eyre::eyre!("{}", error))
            } else {
                Ok::<i32, color_eyre::eyre::Report>(42)
            }
        },
        &config,
        "test operation",
    );

    // 应该重试并成功
    assert!(result.is_ok() || attempt_count.load(Ordering::SeqCst) > 1);
}

// ==================== RetryResult 测试 ====================

#[test]
fn test_retry_result_structure() -> Result<()> {
    let config = HttpRetryConfig {
        max_retries: 3,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let result: RetryResult<i32> = HttpRetry::retry(
        || Ok::<i32, color_eyre::eyre::Report>(42),
        &config,
        "test operation",
    )?;

    assert_eq!(result.result, 42);
    assert_eq!(result.retry_count, 0);
    assert!(result.succeeded_on_first_attempt);
    Ok(())
}
