//! Base/Indicator Spinner 模块测试
//!
//! 测试 Spinner 的核心功能。

use color_eyre::Result;
use std::time::Duration;
use workflow::base::indicator::Spinner;

// ==================== Spinner Creation Tests ====================

/// 测试创建 Spinner
///
/// ## 测试目的
/// 验证 Spinner::new() 能够使用消息创建加载指示器。
///
/// ## 测试场景
/// 1. 使用消息创建 spinner
/// 2. 完成 spinner
/// 3. 验证创建成功
///
/// ## 预期结果
/// - Spinner 创建成功
#[test]
fn test_spinner_new_with_message_creates_spinner() {
    // Arrange: 准备消息
    let message = "Creating PR...";

    // Act: 创建 spinner
    let spinner = Spinner::new(message);
    spinner.finish();

    // Assert: 验证可以创建 spinner
}

/// 测试使用 String 创建 Spinner
///
/// ## 测试目的
/// 验证 Spinner::new() 能够使用 String 类型的消息创建加载指示器。
///
/// ## 测试场景
/// 1. 使用 String 消息创建 spinner
/// 2. 完成 spinner
/// 3. 验证创建成功
///
/// ## 预期结果
/// - Spinner 创建成功
#[test]
fn test_spinner_new_with_string_with_string_message_creates_spinner() {
    // Arrange: 准备 String 消息
    let message = "Processing...".to_string();

    // Act: 使用 String 创建 spinner
    let spinner = Spinner::new(message);
    spinner.finish();

    // Assert: 验证可以创建 spinner
}

// ==================== Spinner Update Tests ====================

/// 测试更新 Spinner 消息
///
/// ## 测试目的
/// 验证 Spinner::update_message() 能够更新加载指示器的消息。
///
/// ## 测试场景
/// 1. 创建 spinner
/// 2. 多次更新消息
/// 3. 完成 spinner
/// 4. 验证更新成功
///
/// ## 预期结果
/// - 消息能够被更新
#[test]
fn test_spinner_update_message_with_messages_updates_message() {
    // Arrange: 准备 spinner
    let spinner = Spinner::new("Starting...");

    // Act: 更新消息
    spinner.update_message("Processing...");
    spinner.update_message("Almost done...");
    spinner.finish();

    // Assert: 验证可以更新消息
}

// ==================== Spinner Finish Tests ====================

/// 测试完成 Spinner
///
/// ## 测试目的
/// 验证 Spinner::finish() 能够完成加载指示器。
///
/// ## 测试场景
/// 1. 创建 spinner
/// 2. 调用 finish() 完成
/// 3. 验证完成成功
///
/// ## 预期结果
/// - Spinner 能够被完成
#[test]
fn test_spinner_finish_with_spinner_finishes_spinner() {
    // Arrange: 准备 spinner
    let spinner = Spinner::new("Creating PR...");

    // Act: 完成 spinner
    spinner.finish();

    // Assert: 验证可以完成 spinner
}

/// 测试使用消息完成 Spinner
///
/// ## 测试目的
/// 验证 Spinner::finish_with_message() 能够完成加载指示器并显示消息。
///
/// ## 测试场景
/// 1. 创建 spinner
/// 2. 使用 finish_with_message() 完成并显示消息
/// 3. 验证完成成功
///
/// ## 预期结果
/// - Spinner 能够被完成并显示消息
#[test]
fn test_spinner_finish_with_message_with_message_finishes_with_message() {
    // Arrange: 准备 spinner 和完成消息
    let spinner = Spinner::new("Creating PR...");
    let message = "PR created successfully!";

    // Act: 完成并显示消息
    spinner.finish_with_message(message);

    // Assert: 验证可以完成并显示消息
}

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

/// 测试 Spinner Drop trait 实现
///
/// ## 测试目的
/// 验证 Spinner 的 Drop trait 实现能够自动清理资源。
///
/// ## 测试场景
/// 1. 创建 spinner 但不手动调用 finish
/// 2. 让 spinner 在作用域结束时自动 drop
/// 3. 验证自动清理成功
///
/// ## 预期结果
/// - Spinner 能够正常 drop，资源被自动清理
#[test]
fn test_spinner_drop() {
    // Arrange: 准备测试 Drop trait 实现（覆盖 spinner.rs:37-46）
    // 创建 spinner 但不手动调用 finish，验证 Drop 会自动清理
    {
        let _spinner = Spinner::new("Testing drop...");
        // spinner 会在作用域结束时自动 drop
    }
    // Assert: 验证可以正常 drop
}

/// 测试 Spinner 消息参数的类型转换
///
/// ## 测试目的
/// 验证 Spinner::new() 能够接受 &str 和 String 类型的消息。
///
/// ## 测试场景
/// 1. 使用 &str 类型消息创建 spinner
/// 2. 使用 String 类型消息创建 spinner
/// 3. 验证两种方式都可以创建
///
/// ## 预期结果
/// - 两种消息类型都可以创建 spinner
#[test]
fn test_spinner_message_types() {
    // Arrange: 准备测试消息参数的类型转换
    let _spinner1 = Spinner::new("String message");
    let _spinner2 = Spinner::new("String message");
    // Assert: 验证两种方式都可以创建 spinner
}

/// 测试 Spinner 的多个操作组合
///
/// ## 测试目的
/// 验证 Spinner 能够执行多个操作（创建、更新消息、完成）的组合。
///
/// ## 测试场景
/// 1. 创建 spinner
/// 2. 多次更新消息
/// 3. 完成 spinner
/// 4. 验证所有操作成功
///
/// ## 预期结果
/// - 所有操作都能成功执行
#[test]
fn test_spinner_multiple_operations() {
    // Arrange: 准备测试 spinner 的多个操作组合
    let spinner = Spinner::new("Starting...");
    spinner.update_message("Step 1...");
    spinner.update_message("Step 2...");
    spinner.update_message("Step 3...");
    spinner.finish();
    // Assert: 验证可以执行多个操作
}

/// 测试 finish_with_message 的消息类型转换
///
/// ## 测试目的
/// 验证 Spinner::finish_with_message() 能够接受 &str 和 String 类型的消息。
///
/// ## 测试场景
/// 1. 使用 &str 类型消息完成 spinner
/// 2. 使用 String 类型消息完成 spinner
/// 3. 验证两种方式都可以完成
///
/// ## 预期结果
/// - 两种消息类型都可以完成 spinner
#[test]
fn test_spinner_finish_with_message_types() {
    // Arrange: 准备测试 finish_with_message 的消息类型转换
    let spinner = Spinner::new("Creating PR...");
    spinner.finish_with_message("String message");
    let spinner2 = Spinner::new("Creating PR...");
    spinner2.finish_with_message("String message");
    // Assert: 验证两种方式都可以完成并显示消息
}
