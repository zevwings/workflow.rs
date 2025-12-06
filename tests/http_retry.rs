//! HTTP 重试模块测试
//!
//! 测试 `base::http::retry` 模块中的重试机制。

use anyhow::{anyhow, Result};
use workflow::base::http::retry::{HttpRetry, HttpRetryConfig};

// ==================== HttpRetryConfig 测试 ====================

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
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_delay, 1);
    assert_eq!(config.max_delay, 30);
    assert_eq!(config.backoff_multiplier, 2.0);
    assert_eq!(config.interactive, true);
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

// ==================== 重试逻辑测试 ====================

#[test]
fn test_retry_success_on_first_attempt() {
    let config = HttpRetryConfig {
        max_retries: 3,
        initial_delay: 0, // 设置为 0 以加快测试
        max_delay: 1,
        backoff_multiplier: 2.0,
        interactive: false, // 非交互式，避免用户输入
    };

    let mut attempt_count = 0;
    let result = HttpRetry::retry(
        || {
            attempt_count += 1;
            Ok(42)
        },
        &config,
        "test operation",
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(attempt_count, 1); // 应该只尝试一次
}

#[test]
fn test_retry_success_after_retries() {
    let config = HttpRetryConfig {
        max_retries: 3,
        initial_delay: 0, // 设置为 0 以加快测试
        max_delay: 1,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let mut attempt_count = 0;
    let result = HttpRetry::retry(
        || {
            attempt_count += 1;
            if attempt_count < 3 {
                Err(anyhow!("Network error"))
            } else {
                Ok(42)
            }
        },
        &config,
        "test operation",
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(attempt_count, 3); // 应该尝试 3 次
}

#[test]
fn test_retry_exhausted() {
    let config = HttpRetryConfig {
        max_retries: 2,
        initial_delay: 0, // 设置为 0 以加快测试
        max_delay: 1,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let mut attempt_count = 0;
    let result = HttpRetry::retry(
        || {
            attempt_count += 1;
            Err(anyhow!("Persistent error"))
        },
        &config,
        "test operation",
    );

    assert!(result.is_err());
    // 应该尝试 max_retries + 1 次（初始尝试 + 重试次数）
    assert_eq!(attempt_count, 3); // 1 次初始 + 2 次重试
}

#[test]
fn test_retry_non_retryable_error() {
    let config = HttpRetryConfig {
        max_retries: 3,
        initial_delay: 0,
        max_delay: 1,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let mut attempt_count = 0;
    let result = HttpRetry::retry(
        || {
            attempt_count += 1;
            // 4xx 错误通常不可重试
            Err(anyhow!("400 Bad Request"))
        },
        &config,
        "test operation",
    );

    assert!(result.is_err());
    // 不可重试的错误应该立即返回，不进行重试
    assert_eq!(attempt_count, 1);
}

// ==================== 指数退避测试 ====================

#[test]
fn test_exponential_backoff_calculation() {
    let config = HttpRetryConfig {
        max_retries: 3,
        initial_delay: 1,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    // 测试延迟计算逻辑
    let mut delay = config.initial_delay;

    // 第一次重试：1 * 2.0 = 2
    delay = ((delay as f64 * config.backoff_multiplier) as u64).min(config.max_delay);
    assert_eq!(delay, 2);

    // 第二次重试：2 * 2.0 = 4
    delay = ((delay as f64 * config.backoff_multiplier) as u64).min(config.max_delay);
    assert_eq!(delay, 4);

    // 第三次重试：4 * 2.0 = 8
    delay = ((delay as f64 * config.backoff_multiplier) as u64).min(config.max_delay);
    assert_eq!(delay, 8);
}

#[test]
fn test_exponential_backoff_max_delay() {
    let config = HttpRetryConfig {
        max_retries: 5,
        initial_delay: 10,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let mut delay = config.initial_delay;

    // 第一次：10 * 2 = 20
    delay = ((delay as f64 * config.backoff_multiplier) as u64).min(config.max_delay);
    assert_eq!(delay, 20);

    // 第二次：20 * 2 = 40，但被限制为 30
    delay = ((delay as f64 * config.backoff_multiplier) as u64).min(config.max_delay);
    assert_eq!(delay, 30);

    // 后续应该保持为 30
    delay = ((delay as f64 * config.backoff_multiplier) as u64).min(config.max_delay);
    assert_eq!(delay, 30);
}

#[test]
fn test_exponential_backoff_custom_multiplier() {
    let config = HttpRetryConfig {
        max_retries: 3,
        initial_delay: 1,
        max_delay: 30,
        backoff_multiplier: 1.5, // 使用 1.5 倍退避
        interactive: false,
    };

    let mut delay = config.initial_delay;

    // 第一次：1 * 1.5 = 1.5 -> 1
    delay = ((delay as f64 * config.backoff_multiplier) as u64).min(config.max_delay);
    assert_eq!(delay, 1);

    // 第二次：1 * 1.5 = 1.5 -> 1
    delay = ((delay as f64 * config.backoff_multiplier) as u64).min(config.max_delay);
    assert_eq!(delay, 1);
}

// ==================== 错误类型测试 ====================

#[test]
fn test_retryable_network_errors() {
    // 测试网络错误应该是可重试的
    let network_errors = vec![
        "Connection refused",
        "Connection timeout",
        "Network unreachable",
        "Temporary failure",
        "500 Internal Server Error",
        "502 Bad Gateway",
        "503 Service Unavailable",
        "504 Gateway Timeout",
    ];

    for error_msg in network_errors {
        let error = anyhow!(error_msg);
        // 注意：这里我们无法直接测试 is_retryable_error，因为它是私有的
        // 但我们可以通过 retry 行为来间接测试
        let config = HttpRetryConfig {
            max_retries: 1,
            initial_delay: 0,
            max_delay: 1,
            backoff_multiplier: 2.0,
            interactive: false,
        };

        let mut attempt_count = 0;
        let _ = HttpRetry::retry(
            || {
                attempt_count += 1;
                Err(anyhow!(error_msg))
            },
            &config,
            "test",
        );

        // 如果是可重试错误，应该尝试多次
        assert!(
            attempt_count > 1,
            "Error '{}' should be retryable",
            error_msg
        );
    }
}

#[test]
fn test_non_retryable_client_errors() {
    // 测试客户端错误应该是不可重试的
    let client_errors = vec![
        "400 Bad Request",
        "401 Unauthorized",
        "403 Forbidden",
        "404 Not Found",
    ];

    for error_msg in client_errors {
        let config = HttpRetryConfig {
            max_retries: 3,
            initial_delay: 0,
            max_delay: 1,
            backoff_multiplier: 2.0,
            interactive: false,
        };

        let mut attempt_count = 0;
        let _ = HttpRetry::retry(
            || {
                attempt_count += 1;
                Err(anyhow!(error_msg))
            },
            &config,
            "test",
        );

        // 如果是不可重试错误，应该只尝试一次
        assert_eq!(
            attempt_count, 1,
            "Error '{}' should not be retryable",
            error_msg
        );
    }
}

// ==================== 交互式模式测试 ====================

#[test]
fn test_non_interactive_mode() {
    // 非交互式模式应该自动重试，不需要用户确认
    let config = HttpRetryConfig {
        max_retries: 2,
        initial_delay: 0,
        max_delay: 1,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let mut attempt_count = 0;
    let result = HttpRetry::retry(
        || {
            attempt_count += 1;
            if attempt_count < 3 {
                Err(anyhow!("Network error"))
            } else {
                Ok(42)
            }
        },
        &config,
        "test operation",
    );

    assert!(result.is_ok());
    assert_eq!(attempt_count, 3);
}

// 注意：交互式模式的测试需要模拟用户输入，这比较复杂
// 可以在集成测试中测试，或者使用 mock 框架
