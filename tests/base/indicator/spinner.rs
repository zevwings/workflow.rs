//! Base/Indicator Spinner 模块测试
//!
//! 测试 Spinner 的核心功能。

use color_eyre::Result;
use std::time::Duration;
use workflow::base::indicator::Spinner;

/// 测试 Spinner::with() 方法成功场景
///
/// ## 测试目的
/// 验证 Spinner::with() 能够在快速操作（< 100ms）时正确执行并返回结果。
///
/// ## 测试场景
/// 1. 使用 with() 方法执行快速操作
/// 2. 验证返回成功结果
///
/// ## 预期结果
/// - 操作成功，返回正确的结果值
#[test]
fn test_spinner_with_success() -> Result<()> {
    // Arrange: 准备测试 with 方法成功场景（覆盖 spinner.rs:175-194）
    let result: Result<i32, Box<dyn std::error::Error>> = Spinner::with("Creating PR...", || {
        // 模拟快速操作（< 100ms）
        Ok(42)
    });
    assert!(result.is_ok());
    let value =
        result.map_err(|e| color_eyre::eyre::eyre!("spinner operation should succeed: {}", e))?;
    assert_eq!(value, 42);
    Ok(())
}

/// 测试 Spinner::with() 方法错误场景
///
/// ## 测试目的
/// 验证 Spinner::with() 在操作失败时能够正确处理错误。
///
/// ## 测试场景
/// 1. 使用 with() 方法执行失败操作
/// 2. 验证返回错误
///
/// ## 预期结果
/// - 操作失败，返回错误信息
#[test]
fn test_spinner_with_error() {
    // Arrange: 准备测试 with 方法错误场景
    let result: Result<i32, String> =
        Spinner::with("Creating PR...", || Err("Operation failed".to_string()));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Operation failed");
}

/// 测试 Spinner::with() 方法慢速操作
///
/// ## 测试目的
/// 验证 Spinner::with() 在慢速操作（> 100ms）时能够显示加载指示器。
///
/// ## 测试场景
/// 1. 使用 with() 方法执行慢速操作
/// 2. 验证操作成功且加载指示器显示
///
/// ## 预期结果
/// - 操作成功，加载指示器在慢速操作时显示
#[test]
fn test_spinner_with_slow_operation() -> Result<()> {
    // Arrange: 准备测试 with 方法慢速操作（> 100ms）
    let result: Result<i32, Box<dyn std::error::Error>> = Spinner::with("Creating PR...", || {
        // 模拟慢速操作（> 100ms）
        std::thread::sleep(Duration::from_millis(150));
        Ok(42)
    });
    assert!(result.is_ok());
    let value = result
        .map_err(|e| color_eyre::eyre::eyre!("slow spinner operation should succeed: {}", e))?;
    assert_eq!(value, 42);
    Ok(())
}

/// 测试 Spinner::with_output() 方法成功场景
///
/// ## 测试目的
/// 验证 Spinner::with_output() 能够执行操作并返回成功结果。
///
/// ## 测试场景
/// 1. 使用 with_output() 方法执行操作
/// 2. 验证返回成功结果
///
/// ## 预期结果
/// - 操作成功，返回正确的结果值
#[test]
fn test_spinner_with_output_success() -> Result<()> {
    // Arrange: 准备测试 with_output 方法成功场景（覆盖 spinner.rs:231-242）
    let result: Result<i32, Box<dyn std::error::Error>> =
        Spinner::with_output("Pushing to remote...", || Ok(42));
    assert!(result.is_ok());
    let value =
        result.map_err(|e| color_eyre::eyre::eyre!("spinner with output should succeed: {}", e))?;
    assert_eq!(value, 42);
    Ok(())
}

/// 测试 Spinner::with_output() 方法错误场景
///
/// ## 测试目的
/// 验证 Spinner::with_output() 在操作失败时能够正确处理错误。
///
/// ## 测试场景
/// 1. 使用 with_output() 方法执行失败操作
/// 2. 验证返回错误
///
/// ## 预期结果
/// - 操作失败，返回错误信息
#[test]
fn test_spinner_with_output_error() {
    // Arrange: 准备测试 with_output 方法错误场景
    let result: Result<i32, String> =
        Spinner::with_output("Pushing to remote...", || Err("Push failed".to_string()));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Push failed");
}
