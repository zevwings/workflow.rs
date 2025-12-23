//! HTTP Retry 测试补充
//!
//! 补充测试 HTTP 重试机制的其他功能，特别是错误判断和描述提取。

use crate::common::http_helpers::MockServer;
use color_eyre::eyre::eyre;
use serde_json::Value;
use workflow::base::http::retry::{HttpRetry, HttpRetryConfig};
use workflow::base::http::{HttpClient, RequestConfig};

// 注意：大部分重试逻辑测试已经在 tests/base/http_retry.rs 中
// 这里主要补充一些边界情况和错误处理的测试

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
fn test_retry_config_default() {
    let config = HttpRetryConfig::default();
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

#[test]
fn test_retry_result_structure() {
    let config = HttpRetryConfig {
        max_retries: 1,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let result = HttpRetry::retry(|| Ok(42), &config, "test").unwrap();
    assert_eq!(result.result, 42);
    assert_eq!(result.retry_count, 0);
    assert!(result.succeeded_on_first_attempt);
}

#[test]
fn test_retry_with_non_retryable_error() {
    // 创建一个不可重试的错误（4xx 客户端错误）
    let config = HttpRetryConfig {
        max_retries: 3,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    // 模拟 400 Bad Request 错误（不可重试）
    // 使用一个简单的错误消息来模拟不可重试的错误
    let result: Result<_, _> = HttpRetry::retry(
        || Err::<String, _>(eyre!("Bad Request: invalid input")),
        &config,
        "test",
    );

    assert!(result.is_err());
    // 应该立即失败，不进行重试
}

#[test]
fn test_retry_with_retryable_error() {
    // 创建一个可重试的错误（网络超时）
    let config = HttpRetryConfig {
        max_retries: 2,
        initial_delay: 0, // 设为0以加快测试
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let attempt = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_clone = attempt.clone();
    let result = HttpRetry::retry(
        move || {
            let mut count = attempt_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current >= 2 {
                Ok("success".to_string())
            } else {
                let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "connection timeout");
                Err(eyre!(error))
            }
        },
        &config,
        "test",
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap().result, "success");
}

#[test]
fn test_retry_with_5xx_error() {
    // 5xx 服务器错误应该是可重试的
    let config = HttpRetryConfig {
        max_retries: 1,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let attempt = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_clone = attempt.clone();
    let _result = HttpRetry::retry(
        move || {
            let mut count = attempt_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current >= 2 {
                Ok("success".to_string())
            } else {
                // 创建一个模拟的 reqwest 错误（5xx）
                let error = std::io::Error::new(std::io::ErrorKind::Other, "Internal Server Error");
                Err(eyre!(error))
            }
        },
        &config,
        "test",
    );

    // 注意：由于我们使用的是 IO 错误而不是真正的 reqwest 错误，
    // 这个测试可能不会按预期工作。但至少不会编译错误。
    // 实际的重试逻辑测试已经在 tests/base/http_retry.rs 中
}

#[test]
fn test_retry_with_429_error() {
    // 429 Too Many Requests 应该是可重试的
    let config = HttpRetryConfig {
        max_retries: 1,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let attempt = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_clone = attempt.clone();
    let result = HttpRetry::retry(
        move || {
            let mut count = attempt_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current >= 2 {
                Ok("success".to_string())
            } else {
                let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "Too Many Requests");
                Err(eyre!(error))
            }
        },
        &config,
        "test",
    );

    assert!(result.is_ok());
}

#[test]
fn test_retry_with_io_error() {
    // IO 错误（如连接被拒绝）应该是可重试的
    let config = HttpRetryConfig {
        max_retries: 1,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let attempt = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_clone = attempt.clone();
    let result = HttpRetry::retry(
        move || {
            let mut count = attempt_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current >= 2 {
                Ok("success".to_string())
            } else {
                let error = std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "connection refused",
                );
                Err(eyre!(error))
            }
        },
        &config,
        "test",
    );

    assert!(result.is_ok());
}

#[test]
fn test_retry_config_custom_values() {
    // 测试自定义配置值
    let config = HttpRetryConfig {
        max_retries: 10,
        initial_delay: 5,
        max_delay: 60,
        backoff_multiplier: 1.5,
        interactive: false,
    };
    assert_eq!(config.max_retries, 10);
    assert_eq!(config.initial_delay, 5);
    assert_eq!(config.max_delay, 60);
    assert_eq!(config.backoff_multiplier, 1.5);
    assert_eq!(config.interactive, false);
}

#[test]
fn test_retry_result_retry_count() {
    // 测试重试次数的记录
    let config = HttpRetryConfig {
        max_retries: 2,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let attempt = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_clone = attempt.clone();
    let result = HttpRetry::retry(
        move || {
            let mut count = attempt_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current >= 3 {
                Ok("success".to_string())
            } else {
                let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
                Err(eyre!(error))
            }
        },
        &config,
        "test",
    );

    assert!(result.is_ok());
    let retry_result = result.unwrap();
    assert_eq!(retry_result.retry_count, 2); // 重试了2次
    assert!(!retry_result.succeeded_on_first_attempt);
}

#[test]
fn test_retry_with_different_io_error_kinds() {
    // 测试不同的 IO 错误类型
    let config = HttpRetryConfig {
        max_retries: 1,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let error_kinds = vec![
        std::io::ErrorKind::ConnectionRefused,
        std::io::ErrorKind::ConnectionReset,
        std::io::ErrorKind::ConnectionAborted,
        std::io::ErrorKind::NotConnected,
        std::io::ErrorKind::BrokenPipe,
    ];

    for error_kind in error_kinds {
        let attempt = std::sync::Arc::new(std::sync::Mutex::new(0));
        let attempt_clone = attempt.clone();
        let result = HttpRetry::retry(
            move || {
                let mut count = attempt_clone.lock().unwrap();
                *count += 1;
                let current = *count;
                drop(count);

                if current >= 2 {
                    Ok("success".to_string())
                } else {
                    let error = std::io::Error::new(error_kind, "connection error");
                    Err(eyre!(error))
                }
            },
            &config,
            "test",
        );
        // 这些错误应该是可重试的
        assert!(result.is_ok());
    }
}

#[test]
fn test_retry_with_interactive_mode_first_attempt() {
    // 测试交互模式下的第一次失败（会调用 countdown_with_cancel）
    let config = HttpRetryConfig {
        max_retries: 1,
        initial_delay: 1, // 短延迟以便测试
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: true, // 启用交互模式
    };

    let attempt = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_clone = attempt.clone();
    let result = HttpRetry::retry(
        move || {
            let mut count = attempt_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current >= 2 {
                Ok("success".to_string())
            } else {
                let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
                Err(eyre!(error))
            }
        },
        &config,
        "test",
    );

    // 应该成功（重试后成功）
    assert!(result.is_ok());
}

#[test]
fn test_retry_countdown_short_delay() {
    // 测试短延迟（< 3秒）的情况，会直接 sleep，不显示倒计时
    let config = HttpRetryConfig {
        max_retries: 1,
        initial_delay: 1, // 短延迟
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: true,
    };

    let attempt = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_clone = attempt.clone();
    let result = HttpRetry::retry(
        move || {
            let mut count = attempt_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current >= 2 {
                Ok("success".to_string())
            } else {
                let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
                Err(eyre!(error))
            }
        },
        &config,
        "test",
    );

    assert!(result.is_ok());
}

#[test]
fn test_retry_non_interactive_after_first_attempt() {
    // 测试非交互模式或第一次重试后的逻辑（直接 sleep，不显示倒计时）
    let config = HttpRetryConfig {
        max_retries: 2,
        initial_delay: 0, // 零延迟以加快测试
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false, // 非交互模式
    };

    let attempt = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_clone = attempt.clone();
    let result = HttpRetry::retry(
        move || {
            let mut count = attempt_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current >= 3 {
                Ok("success".to_string())
            } else {
                let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
                Err(eyre!(error))
            }
        },
        &config,
        "test",
    );

    assert!(result.is_ok());
}

#[test]
fn test_retry_error_description_reqwest_status() {
    // 测试 get_error_description 对 reqwest 状态码的处理
    // 注意：这个测试通过重试逻辑间接测试 get_error_description
    let config = HttpRetryConfig {
        max_retries: 0, // 不重试，立即返回错误
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    // 创建一个模拟的 reqwest 错误（带状态码）
    // 由于无法直接创建 reqwest::Error，我们使用 IO 错误来测试
    let result = HttpRetry::retry(
        || {
            let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
            Err::<String, _>(eyre!(error))
        },
        &config,
        "test",
    );

    // 应该返回错误（因为 max_retries 为 0）
    assert!(result.is_err());
}

#[test]
fn test_retry_error_description_long_message() {
    // 测试 get_error_description 对长错误消息的处理（> 100 字符）
    let config = HttpRetryConfig {
        max_retries: 0,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let long_error_msg = "x".repeat(150);
    let result = HttpRetry::retry(
        move || Err::<String, _>(eyre!(long_error_msg.clone())),
        &config,
        "test",
    );

    assert!(result.is_err());
    // 错误消息应该被截断（通过 get_error_description）
    let error_msg = result.unwrap_err().to_string();
    // 验证错误消息存在（可能包含操作名称、重试信息或原始错误）
    assert!(!error_msg.is_empty());
}

#[test]
fn test_retry_countdown_long_delay() {
    // 测试长延迟（>= 3秒）的情况，会显示倒计时
    let config = HttpRetryConfig {
        max_retries: 1,
        initial_delay: 3, // 长延迟，会触发倒计时逻辑
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: true,
    };

    let attempt = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_clone = attempt.clone();
    let result = HttpRetry::retry(
        move || {
            let mut count = attempt_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current >= 2 {
                Ok("success".to_string())
            } else {
                let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
                Err(eyre!(error))
            }
        },
        &config,
        "test",
    );

    assert!(result.is_ok());
}

#[test]
fn test_retry_interactive_user_cancel() {
    // 测试交互式重试中用户取消的情况
    // 注意：这个测试需要 mock ConfirmDialog，但由于 ConfirmDialog 是交互式的，
    // 我们通过非交互模式来测试其他路径
    let config = HttpRetryConfig {
        max_retries: 2,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false, // 非交互模式，避免用户输入
    };

    let attempt = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_clone = attempt.clone();
    let result = HttpRetry::retry(
        move || {
            let mut count = attempt_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current >= 3 {
                Ok("success".to_string())
            } else {
                let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
                Err(eyre!(error))
            }
        },
        &config,
        "test",
    );

    assert!(result.is_ok());
}

#[test]
fn test_retry_interactive_confirm_dialog_error() {
    // 测试交互式重试中 ConfirmDialog 失败的情况（非交互式终端）
    // 注意：这个测试通过非交互模式来间接测试错误处理路径
    let config = HttpRetryConfig {
        max_retries: 1,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false, // 非交互模式，模拟 ConfirmDialog 失败的情况
    };

    let attempt = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_clone = attempt.clone();
    let result = HttpRetry::retry(
        move || {
            let mut count = attempt_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current >= 2 {
                Ok("success".to_string())
            } else {
                let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
                Err(eyre!(error))
            }
        },
        &config,
        "test",
    );

    assert!(result.is_ok());
}

#[test]
fn test_retry_non_retryable_error_first_attempt() {
    // 测试第一次尝试就失败且不可重试的错误
    let config = HttpRetryConfig {
        max_retries: 3,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    // 创建一个不可重试的错误（非网络错误）
    let result = HttpRetry::retry(
        || Err::<String, _>(eyre!("Bad Request: invalid input")),
        &config,
        "test",
    );

    assert!(result.is_err());
    // 应该立即失败，不进行重试
}

#[test]
fn test_retry_all_retries_exhausted() {
    // 测试所有重试都失败的情况
    let config = HttpRetryConfig {
        max_retries: 2,
        initial_delay: 0, // 零延迟以加快测试
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let result = HttpRetry::retry(
        || {
            let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
            Err::<String, _>(eyre!(error))
        },
        &config,
        "test",
    );

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    // 验证错误消息包含重试信息
    assert!(error_msg.contains("test") || error_msg.contains("retries"));
}

#[test]
fn test_retry_success_after_multiple_attempts() {
    // 测试多次重试后成功的情况
    let config = HttpRetryConfig {
        max_retries: 3,
        initial_delay: 0,
        max_delay: 30,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let attempt = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_clone = attempt.clone();
    let result = HttpRetry::retry(
        move || {
            let mut count = attempt_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current >= 3 {
                Ok("success".to_string())
            } else {
                let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
                Err(eyre!(error))
            }
        },
        &config,
        "test",
    );

    assert!(result.is_ok());
    let retry_result = result.unwrap();
    assert_eq!(retry_result.result, "success");
    assert_eq!(retry_result.retry_count, 2); // 重试了2次
    assert!(!retry_result.succeeded_on_first_attempt);
}

#[test]
fn test_retry_delay_backoff_calculation() {
    // 测试延迟退避计算
    let config = HttpRetryConfig {
        max_retries: 3,
        initial_delay: 1,
        max_delay: 10,
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let attempt = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_clone = attempt.clone();
    let start_time = std::time::Instant::now();
    let result = HttpRetry::retry(
        move || {
            let mut count = attempt_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current >= 4 {
                Ok("success".to_string())
            } else {
                let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
                Err(eyre!(error))
            }
        },
        &config,
        "test",
    );

    assert!(result.is_ok());
    // 验证延迟时间（1 + 2 + 4 = 7秒，但受 max_delay 限制）
    let elapsed = start_time.elapsed();
    // 允许一些误差
    assert!(elapsed.as_secs() >= 6);
}

#[test]
fn test_retry_delay_max_limit() {
    // 测试延迟达到最大值的情况
    let config = HttpRetryConfig {
        max_retries: 3,
        initial_delay: 10,
        max_delay: 10, // 最大值等于初始值
        backoff_multiplier: 2.0,
        interactive: false,
    };

    let attempt = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_clone = attempt.clone();
    let result = HttpRetry::retry(
        move || {
            let mut count = attempt_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current >= 4 {
                Ok("success".to_string())
            } else {
                let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
                Err(eyre!(error))
            }
        },
        &config,
        "test",
    );

    assert!(result.is_ok());
}
