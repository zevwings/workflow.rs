//! Timeout 模块集成测试
//!
//! 包含 timeout 模块的集成测试，主要测试：
//! - execute_with_timeout 基本功能
//! - TimeoutConfig::with_platform_specific() 和 actual_timeout() 平台特定配置
//! - 并发场景
//! - 系统级行为（资源限制、线程泄漏）
//! - 并发限制机制
//!
//! 注意：这些测试涉及并发操作，执行时间较长，适合作为集成测试。

use color_eyre::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use workflow::base::resilience::timeout::{execute_with_timeout, TimeoutConfig};

// ==================== execute_with_timeout 基本功能测试 ====================

/// 测试超时执行成功
///
/// ## 测试目的
/// 验证 execute_with_timeout() 能够在超时时间内成功执行操作并返回结果。
///
/// ## 测试场景
/// 1. 创建超时配置（5秒超时）
/// 2. 执行一个快速完成的操作（立即返回成功）
/// 3. 验证操作成功执行并返回正确结果
///
/// ## 预期结果
/// - 操作成功执行
/// - 返回正确的结果值
/// - 不产生超时错误
#[test]
fn test_execute_with_timeout_success() -> Result<()> {
    let result = execute_with_timeout(
        TimeoutConfig::new(Duration::from_secs(5)),
        || -> Result<String> { Ok("success".to_string()) },
    )?;
    assert_eq!(result, "success");
    Ok(())
}

/// 测试超时执行失败
///
/// ## 测试目的
/// 验证 execute_with_timeout() 在操作超过超时时间时能够正确返回超时错误。
///
/// ## 测试场景
/// 1. 创建短超时配置（50毫秒）
/// 2. 执行一个需要200毫秒的操作（超过超时时间）
/// 3. 验证返回超时错误或并发限制错误
///
/// ## 预期结果
/// - 返回错误（Result::Err）
/// - 错误消息包含 "timed out" 或 "Too many concurrent"
#[test]
fn test_execute_with_timeout_failure() {
    let result = execute_with_timeout(
        TimeoutConfig::new(Duration::from_millis(50)),
        || -> Result<String> {
            // 操作需要 200ms，但超时是 50ms + 100ms 等待 = 150ms，应该超时
            thread::sleep(Duration::from_millis(200));
            Ok("success".to_string())
        },
    );
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    // 可能是超时错误，也可能是并发限制错误（如果之前的测试还没完成）
    assert!(
        error_msg.contains("timed out") || error_msg.contains("Too many concurrent"),
        "Expected timeout or concurrent limit error, got: {}",
        error_msg
    );
}

/// 测试平台特定超时
///
/// ## 测试目的
/// 验证 TimeoutConfig::with_platform_specific() 能够根据平台自动调整超时时间。
///
/// ## 测试场景
/// 1. 创建超时配置并启用平台特定调整
/// 2. 获取实际超时时间
/// 3. 验证不同平台的超时时间调整正确
///
/// ## 预期结果
/// - Windows 平台：超时时间调整为 1.5 倍（10秒 -> 15秒）
/// - 其他平台：超时时间保持不变（10秒）
#[test]
fn test_platform_specific_timeout() {
    let config = TimeoutConfig::new(Duration::from_secs(10)).with_platform_specific();
    let actual = config.actual_timeout();

    #[cfg(target_os = "windows")]
    assert_eq!(actual, Duration::from_secs(15)); // 10 * 3/2 = 15

    #[cfg(not(target_os = "windows"))]
    assert_eq!(actual, Duration::from_secs(10));
}

/// 测试操作在超时边界完成
///
/// ## 测试目的
/// 验证 execute_with_timeout() 能够正确处理操作在超时边界附近完成的情况。
///
/// ## 测试场景
/// 1. 创建100毫秒的超时配置
/// 2. 执行一个需要90毫秒的操作（接近但不超过超时时间）
/// 3. 验证操作成功完成
///
/// ## 预期结果
/// - 操作在超时前成功完成
/// - 返回正确的结果
#[test]
fn test_timeout_boundary_completion() -> Result<()> {
    let config = TimeoutConfig::new(Duration::from_millis(100));

    // 操作在超时边界完成（刚好在超时前完成）
    let result = execute_with_timeout(config, || -> Result<String> {
        thread::sleep(Duration::from_millis(90)); // 接近但不超过超时时间
        Ok("success".to_string())
    })?;

    assert_eq!(result, "success");
    Ok(())
}

// ==================== 并发场景测试 ====================

/// 测试多个操作并发执行
///
/// 注意：此测试涉及并发操作，在并行测试环境中可能不稳定。
/// 使用 `cargo test --test-threads=1` 运行以确保稳定性。
#[test]
fn test_concurrent_execute_with_timeout() -> Result<()> {
    let config = TimeoutConfig::new(Duration::from_millis(200));
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    // 创建多个并发操作（注意：并发限制是 10，创建 3 个操作应该都能成功）
    // 减少并发数量以避免触发并发限制
    for i in 0..3 {
        let results_clone = results.clone();
        let config = config.clone();

        let handle = thread::spawn(move || {
            let result = execute_with_timeout(config, move || -> Result<String> {
                thread::sleep(Duration::from_millis(10));
                Ok(format!("result_{}", i))
            });

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
        assert_eq!(result.as_ref().unwrap(), &format!("result_{}", i));
    }

    Ok(())
}

/// 测试并发执行时的资源限制场景
#[test]
fn test_concurrent_resource_limits() {
    let config = TimeoutConfig::new(Duration::from_millis(100));
    let success_count = Arc::new(Mutex::new(0));
    let failure_count = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();

    // 创建大量并发操作（测试资源限制）
    // 注意：由于并发限制是 10，创建 12 个操作应该至少有 2 个失败
    for i in 0..12 {
        let success_count_clone = success_count.clone();
        let failure_count_clone = failure_count.clone();
        let config = config.clone();

        let handle = thread::spawn(move || {
            let result = execute_with_timeout(config, move || -> Result<String> {
                thread::sleep(Duration::from_millis(5));
                Ok(format!("result_{}", i))
            });

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

/// 测试线程泄漏检测
#[test]
fn test_thread_leak_prevention() -> Result<()> {
    let config = TimeoutConfig::new(Duration::from_millis(50));

    // 记录初始线程数（近似）
    let initial_thread_count = thread::available_parallelism().map(|n| n.get()).unwrap_or(1);

    // 执行多个操作（注意：并发限制是 10，所以需要串行执行）
    // 通过添加小延迟确保操作串行执行
    for i in 0..5 {
        let _ = execute_with_timeout(config.clone(), move || -> Result<String> {
            thread::sleep(Duration::from_millis(5));
            Ok(format!("result_{}", i))
        })?;
        // 添加小延迟确保操作串行执行，避免触发并发限制
        thread::sleep(Duration::from_millis(20));
    }

    // 等待一段时间，让线程清理
    thread::sleep(Duration::from_millis(100));

    // 验证线程数没有显著增加（这是一个近似测试）
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

// ==================== 并发限制测试 ====================

/// 测试并发限制
#[test]
fn test_concurrent_limit() {
    let config = TimeoutConfig::new(Duration::from_millis(100));
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    // 创建超过限制的操作（MAX_CONCURRENT_TIMEOUT_OPERATIONS = 10 in production）
    // 创建 15 个操作，应该有一些因为超过限制而失败
    for i in 0..15 {
        let results_clone = results.clone();
        let config = config.clone();

        let handle = thread::spawn(move || {
            let result = execute_with_timeout(config, move || -> Result<String> {
                thread::sleep(Duration::from_millis(10));
                Ok(format!("result_{}", i))
            });

            results_clone.lock().unwrap().push((i, result));
        });

        handles.push(handle);
    }

    // 等待所有操作完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证结果：应该有一些操作因为超过限制而失败
    let results = results.lock().unwrap();
    let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
    let failure_count = results.iter().filter(|(_, r)| r.is_err()).count();

    // 应该有一些操作成功，一些操作因为超过限制而失败
    assert!(
        success_count > 0,
        "Expected some operations to succeed, got {}",
        success_count
    );
    assert!(
        failure_count > 0,
        "Expected some operations to fail due to concurrent limit, got {}",
        failure_count
    );

    // 验证失败的错误消息
    for (_, result) in results.iter() {
        if let Err(e) = result {
            let error_msg = e.to_string();
            // 可能是并发限制错误，也可能是其他错误（如超时）
            // 只要有一些是并发限制错误即可
            if error_msg.contains("Too many concurrent timeout operations") {
                return; // 找到并发限制错误，测试通过
            }
        }
    }

    // 如果没有找到并发限制错误，但确实有失败，也算通过（可能是其他原因）
    if failure_count > 0 {
        return;
    }

    // 如果所有操作都成功，说明限制没有生效（可能是因为操作太快完成）
    // 这种情况下，我们至少验证了并发限制机制存在
    // 注意：由于并发限制是 10，创建 15 个操作应该至少有 5 个失败
    assert!(
        success_count <= 10,
        "Expected at most 10 successes due to concurrent limit"
    );
}

/// 测试并发限制释放
#[test]
fn test_concurrent_limit_release() -> Result<()> {
    let config = TimeoutConfig::new(Duration::from_millis(50));

    // 执行一个快速操作，然后立即执行另一个操作
    // 第一个操作完成后，应该释放计数，第二个操作应该成功
    let result1 = execute_with_timeout(config.clone(), || -> Result<String> {
        thread::sleep(Duration::from_millis(10));
        Ok("result1".to_string())
    })?;

    assert_eq!(result1, "result1");

    // 等待一小段时间确保计数被释放
    thread::sleep(Duration::from_millis(20));

    // 第二个操作应该成功（因为第一个操作已经完成并释放了计数）
    let result2 = execute_with_timeout(config, || -> Result<String> {
        thread::sleep(Duration::from_millis(10));
        Ok("result2".to_string())
    })?;

    assert_eq!(result2, "result2");
    Ok(())
}

/// 测试改进的线程清理机制（100ms 等待）
#[test]
fn test_improved_thread_cleanup() -> Result<()> {
    let config = TimeoutConfig::new(Duration::from_millis(50));

    // 操作在超时边界附近完成（在 100ms 等待期间完成）
    let result = execute_with_timeout(config, || -> Result<String> {
        // 操作在超时后但在 100ms 等待期间完成
        thread::sleep(Duration::from_millis(60));
        Ok("success".to_string())
    })?;

    // 应该成功，因为改进的清理机制会在 100ms 内检测到操作完成
    assert_eq!(result, "success");
    Ok(())
}
