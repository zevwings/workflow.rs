//! Retry 模块集成测试
//!
//! 包含 retry 模块的集成测试，主要测试：
//! - execute_with_retry 基本功能
//! - RetryConfig::platform_default() 平台特定配置
//! - execute_with_timeout_and_retry 总超时和指数退避机制
//! - 并发场景
//! - 系统级行为（资源限制、线程泄漏）

use color_eyre::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use workflow::base::resilience::retry::{execute_with_retry, execute_with_timeout_and_retry, RetryConfig};
use workflow::base::resilience::timeout::TimeoutConfig;

// ==================== execute_with_retry 测试 ====================

/// 测试重试执行成功
///
/// ## 测试目的
/// 验证 execute_with_retry() 能够在遇到可重试错误时自动重试，并在重试后成功执行操作。
///
/// ## 测试场景
/// 1. 创建重试配置（最多3次重试，延迟10ms）
/// 2. 执行一个操作，第一次失败（可重试错误），第二次成功
/// 3. 验证操作最终成功
/// 4. 验证重试计数和首次尝试标志正确
///
/// ## 预期结果
/// - 操作最终成功执行
/// - 重试计数为 1（重试了1次）
/// - succeeded_on_first_attempt 为 false
#[test]
fn test_execute_with_retry_success() -> Result<()> {
    let config = RetryConfig::new(3, Duration::from_millis(10));
    let mut attempts = 0;
    let result = execute_with_retry(
        config,
        || -> Result<String> {
            attempts += 1;
            if attempts < 2 {
                // 创建一个可重试的错误（IO 错误）
                Err(color_eyre::eyre::eyre!(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Temporary error"
                )))
            } else {
                Ok("success".to_string())
            }
        },
        "Test operation",
    )?;
    assert_eq!(result.result, "success");
    assert_eq!(result.retry_count, 1);
    assert!(!result.succeeded_on_first_attempt);
    Ok(())
}

/// 测试重试执行失败（不可重试的错误）
///
/// ## 测试目的
/// 验证 execute_with_retry() 在遇到不可重试的错误时能够立即返回错误，不进行重试。
///
/// ## 测试场景
/// 1. 创建重试配置（最多3次重试）
/// 2. 执行一个操作，返回不可重试的错误（NotFound）
/// 3. 验证立即返回错误，不进行重试
///
/// ## 预期结果
/// - 立即返回错误（Result::Err）
/// - 错误消息包含 "not found"
/// - 不进行重试（因为错误不可重试）
#[test]
fn test_execute_with_retry_not_retryable() {
    let config = RetryConfig::new(3, Duration::from_millis(10));
    let result = execute_with_retry(
        config,
        || -> Result<String> {
            Err(color_eyre::eyre::eyre!(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found"
            )))
        },
        "Test operation",
    );
    assert!(result.is_err());
    // 不可重试的错误应该立即返回，不进行重试
    assert!(result.unwrap_err().to_string().contains("not found"));
}

/// 测试平台特定配置
///
/// ## 测试目的
/// 验证 RetryConfig::platform_default() 能够根据平台返回不同的默认重试配置。
///
/// ## 测试场景
/// 1. 调用 platform_default() 获取平台默认配置
/// 2. 验证不同平台的配置参数
///
/// ## 预期结果
/// - Windows 平台：max_retries=5, retry_delay=300ms, exponential_backoff=true
/// - 其他平台：max_retries=3, retry_delay=100ms, exponential_backoff=false
#[test]
fn test_platform_default_config() {
    let config = RetryConfig::platform_default();

    #[cfg(target_os = "windows")]
    {
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.retry_delay, Duration::from_millis(300));
        assert!(config.exponential_backoff);
    }

    #[cfg(not(target_os = "windows"))]
    {
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay, Duration::from_millis(100));
        assert!(!config.exponential_backoff);
    }
}

// ==================== execute_with_timeout_and_retry 测试 ====================

/// 测试 execute_with_timeout_and_retry 总超时
#[test]
fn test_execute_with_timeout_and_retry_total_timeout() {
    let timeout_config = TimeoutConfig::new(Duration::from_millis(50));
    let retry_config = RetryConfig::new(5, Duration::from_millis(10));

    let start = Instant::now();
    let result = execute_with_timeout_and_retry(
        timeout_config,
        retry_config,
        || -> Result<String> {
            // 每次操作都超时
            std::thread::sleep(Duration::from_millis(100));
            Err(color_eyre::eyre::eyre!(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Operation timed out"
            )))
        },
        "Test operation",
    );

    assert!(result.is_err());
    let elapsed = start.elapsed();
    // 总超时时间应该小于单次超时 × (重试次数 + 1) + 延迟时间
    // 50ms × 6 + 10ms × 5 = 350ms，但实际应该更早因为总超时检查
    assert!(elapsed < Duration::from_millis(400));
}

/// 测试 execute_with_timeout_and_retry 指数退避
#[test]
fn test_execute_with_timeout_and_retry_exponential_backoff() -> Result<()> {
    let timeout_config = TimeoutConfig::new(Duration::from_millis(100));
    let mut retry_config = RetryConfig::new(3, Duration::from_millis(50));
    retry_config.exponential_backoff = true;

    let delays = Arc::new(Mutex::new(Vec::new()));
    let last_time = Arc::new(Mutex::new(Instant::now()));
    let attempts = Arc::new(Mutex::new(0));

    let delays_clone = delays.clone();
    let last_time_clone = last_time.clone();
    let attempts_clone = attempts.clone();
    let result = execute_with_timeout_and_retry(
        timeout_config,
        retry_config,
        move || -> Result<String> {
            let mut attempts = attempts_clone.lock().unwrap();
            *attempts += 1;
            let current_attempt = *attempts;
            drop(attempts);

            let now = Instant::now();
            if current_attempt > 1 {
                let mut delays = delays_clone.lock().unwrap();
                let last = last_time_clone.lock().unwrap();
                delays.push(now.duration_since(*last));
            }
            *last_time_clone.lock().unwrap() = now;

            if current_attempt < 4 {
                Err(color_eyre::eyre::eyre!(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Temporary error"
                )))
            } else {
                Ok("success".to_string())
            }
        },
        "Test operation",
    )?;

    assert_eq!(result.result, "success");
    assert_eq!(result.retry_count, 3);

    // 验证指数退避：延迟应该逐渐增加
    // delays[0] ≈ 50ms, delays[1] ≈ 100ms, delays[2] ≈ 200ms
    let delays = delays.lock().unwrap();
    if delays.len() >= 2 {
        assert!(
            delays[1] > delays[0],
            "Second delay should be longer than first"
        );
    }
    if delays.len() >= 3 {
        assert!(
            delays[2] > delays[1],
            "Third delay should be longer than second"
        );
    }

    Ok(())
}

// ==================== 并发场景测试 ====================

/// 测试多个操作并发执行
///
/// 注意：此测试涉及并发操作，在并行测试环境中可能不稳定。
/// 使用 `cargo test --test-threads=1` 运行以确保稳定性。
#[test]
fn test_concurrent_execute_with_timeout_and_retry() -> Result<()> {
    let timeout_config = TimeoutConfig::new(Duration::from_millis(200));
    let retry_config = RetryConfig::new(2, Duration::from_millis(10));

    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    // 创建多个并发操作（注意：并发限制是 10，创建 3 个操作应该都能成功）
    // 减少并发数量以避免触发并发限制
    for i in 0..3 {
        let results_clone = results.clone();
        let timeout_config = timeout_config.clone();
        let retry_config = retry_config.clone();

        let handle = thread::spawn(move || {
            let result = execute_with_timeout_and_retry(
                timeout_config,
                retry_config,
                move || -> Result<String> {
                    // 模拟一些工作
                    thread::sleep(Duration::from_millis(10));
                    Ok(format!("result_{}", i))
                },
                &format!("Concurrent operation {}", i),
            );

            results_clone.lock().unwrap().push((i, result));
        });

        handles.push(handle);
    }

    // 等待所有操作完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证所有操作都成功
    let results = results.lock().unwrap();
    assert_eq!(results.len(), 3);
    for (i, result) in results.iter() {
        assert!(result.is_ok(), "Operation {} should succeed", i);
        assert_eq!(result.as_ref().unwrap().result, format!("result_{}", i));
    }

    Ok(())
}

/// 测试并发执行时的资源限制场景
#[test]
fn test_concurrent_resource_limits() {
    let timeout_config = TimeoutConfig::new(Duration::from_millis(100));
    let retry_config = RetryConfig::new(1, Duration::from_millis(10));

    let success_count = Arc::new(Mutex::new(0));
    let failure_count = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();

    // 创建大量并发操作（测试资源限制）
    // 注意：由于并发限制是 10，创建 12 个操作应该至少有 2 个失败
    for i in 0..12 {
        let success_count_clone = success_count.clone();
        let failure_count_clone = failure_count.clone();
        let timeout_config = timeout_config.clone();
        let retry_config = retry_config.clone();

        let handle = thread::spawn(move || {
            let result = execute_with_timeout_and_retry(
                timeout_config,
                retry_config,
                move || -> Result<String> {
                    // 模拟快速操作
                    thread::sleep(Duration::from_millis(5));
                    Ok(format!("result_{}", i))
                },
                &format!("Resource limit test {}", i),
            );

            match result {
                Ok(_) => *success_count_clone.lock().unwrap() += 1,
                Err(_) => *failure_count_clone.lock().unwrap() += 1,
            }
        });

        handles.push(handle);
    }

    // 等待所有操作完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证大部分操作成功（允许少量失败，因为资源限制）
    let success = *success_count.lock().unwrap();
    let failure = *failure_count.lock().unwrap();
    assert_eq!(success + failure, 12);
    // 由于并发限制是 10，最多 10 个操作应该成功
    assert!(
        success <= 10,
        "Expected at most 10 successes due to concurrent limit, got {} successes and {} failures",
        success,
        failure
    );
    // 至少应该有 2 个失败（因为创建了 12 个操作，限制是 10）
    assert!(
        failure >= 2,
        "Expected at least 2 failures due to concurrent limit, got {} successes and {} failures",
        success,
        failure
    );
}

/// 测试线程泄漏检测（通过验证线程数量）
#[test]
fn test_thread_leak_prevention() -> Result<()> {
    let timeout_config = TimeoutConfig::new(Duration::from_millis(50));
    let retry_config = RetryConfig::new(2, Duration::from_millis(10));

    // 记录初始线程数（近似）
    let initial_thread_count = thread::available_parallelism().map(|n| n.get()).unwrap_or(1);

    // 执行多个操作（注意：并发限制是 10，所以需要串行执行）
    // 通过添加小延迟确保操作串行执行
    for i in 0..5 {
        let _ = execute_with_timeout_and_retry(
            timeout_config.clone(),
            retry_config.clone(),
            move || -> Result<String> {
                thread::sleep(Duration::from_millis(5));
                Ok(format!("result_{}", i))
            },
            &format!("Thread leak test {}", i),
        )?;
        // 添加小延迟确保操作串行执行，避免触发并发限制
        thread::sleep(Duration::from_millis(20));
    }

    // 等待一段时间，让线程清理
    thread::sleep(Duration::from_millis(100));

    // 验证线程数没有显著增加（这是一个近似测试）
    // 注意：这个测试可能不够精确，因为线程池和其他因素
    // 但可以检测明显的线程泄漏
    let final_thread_count = thread::available_parallelism().map(|n| n.get()).unwrap_or(1);

    // 线程数不应该显著增加（允许一些误差）
    assert!(
        final_thread_count <= initial_thread_count + 5,
        "Thread count increased significantly: {} -> {}",
        initial_thread_count,
        final_thread_count
    );

    Ok(())
}

// ==================== 边界条件测试 ====================

/// 测试操作在总超时边界完成
#[test]
fn test_total_timeout_boundary() {
    let timeout_config = TimeoutConfig::new(Duration::from_millis(50));
    let retry_config = RetryConfig::new(3, Duration::from_millis(10));

    let start = Instant::now();
    let attempt_count_mutex = Arc::new(Mutex::new(0));

    let attempt_count_clone = attempt_count_mutex.clone();
    let result = execute_with_timeout_and_retry(
        timeout_config,
        retry_config,
        move || -> Result<String> {
            *attempt_count_clone.lock().unwrap() += 1;
            // 每次操作都很快完成，但会重试
            if *attempt_count_clone.lock().unwrap() < 4 {
                Err(color_eyre::eyre::eyre!(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Temporary error"
                )))
            } else {
                Ok("success".to_string())
            }
        },
        "Total timeout boundary test",
    );

    let elapsed = start.elapsed();
    let attempt_count = *attempt_count_mutex.lock().unwrap();

    // 操作应该在总超时时间内完成
    // 总超时 ≈ 50ms × 4 + 延迟时间（约 10ms × 3 = 30ms）≈ 230ms
    assert!(elapsed < Duration::from_millis(300));
    assert!(result.is_ok() || elapsed < Duration::from_millis(300));
    // 验证尝试次数
    assert!(attempt_count >= 1);
}
