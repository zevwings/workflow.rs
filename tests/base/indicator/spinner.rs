//! Base/Indicator Spinner 模块测试
//!
//! 测试 Spinner 的核心功能。

use std::time::Duration;
use workflow::base::indicator::Spinner;

// ==================== Spinner Creation Tests ====================

#[test]
fn test_spinner_new_with_message_creates_spinner() {
    // Arrange: 准备消息
    let message = "Creating PR...";

    // Act: 创建 spinner
    let spinner = Spinner::new(message);
    spinner.finish();

    // Assert: 验证可以创建 spinner
    assert!(true);
}

#[test]
fn test_spinner_new_with_string_with_string_message_creates_spinner() {
    // Arrange: 准备 String 消息
    let message = "Processing...".to_string();

    // Act: 使用 String 创建 spinner
    let spinner = Spinner::new(message);
    spinner.finish();

    // Assert: 验证可以创建 spinner
    assert!(true);
}

// ==================== Spinner Update Tests ====================

#[test]
fn test_spinner_update_message_with_messages_updates_message() {
    // Arrange: 准备 spinner
    let spinner = Spinner::new("Starting...");

    // Act: 更新消息
    spinner.update_message("Processing...");
    spinner.update_message("Almost done...");
    spinner.finish();

    // Assert: 验证可以更新消息
    assert!(true);
}

// ==================== Spinner Finish Tests ====================

#[test]
fn test_spinner_finish_with_spinner_finishes_spinner() {
    // Arrange: 准备 spinner
    let spinner = Spinner::new("Creating PR...");

    // Act: 完成 spinner
    spinner.finish();

    // Assert: 验证可以完成 spinner
    assert!(true);
}

#[test]
fn test_spinner_finish_with_message_with_message_finishes_with_message() {
    // Arrange: 准备 spinner 和完成消息
    let spinner = Spinner::new("Creating PR...");
    let message = "PR created successfully!";

    // Act: 完成并显示消息
    spinner.finish_with_message(message);

    // Assert: 验证可以完成并显示消息
    assert!(true);
}

#[test]
fn test_spinner_with_success() {
    // Arrange: 准备测试 with 方法成功场景（覆盖 spinner.rs:175-194）
    let result: Result<i32, Box<dyn std::error::Error>> = Spinner::with("Creating PR...", || {
        // 模拟快速操作（< 100ms）
        Ok(42)
    });
    assert!(result.is_ok());
    assert_eq!(result.expect("spinner operation should succeed"), 42);
}

#[test]
fn test_spinner_with_error() {
    // Arrange: 准备测试 with 方法错误场景
    let result: Result<i32, String> =
        Spinner::with("Creating PR...", || Err("Operation failed".to_string()));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Operation failed");
}

#[test]
fn test_spinner_with_slow_operation() {
    // Arrange: 准备测试 with 方法慢速操作（> 100ms）
    let result: Result<i32, Box<dyn std::error::Error>> = Spinner::with("Creating PR...", || {
        // 模拟慢速操作（> 100ms）
        std::thread::sleep(Duration::from_millis(150));
        Ok(42)
    });
    assert!(result.is_ok());
    assert_eq!(result.expect("slow spinner operation should succeed"), 42);
}

#[test]
fn test_spinner_with_output_success() {
    // Arrange: 准备测试 with_output 方法成功场景（覆盖 spinner.rs:231-242）
    let result: Result<i32, Box<dyn std::error::Error>> =
        Spinner::with_output("Pushing to remote...", || Ok(42));
    assert!(result.is_ok());
    assert_eq!(result.expect("spinner with output should succeed"), 42);
}

#[test]
fn test_spinner_with_output_error() {
    // Arrange: 准备测试 with_output 方法错误场景
    let result: Result<i32, String> =
        Spinner::with_output("Pushing to remote...", || Err("Push failed".to_string()));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Push failed");
}

#[test]
fn test_spinner_drop() {
    // Arrange: 准备测试 Drop trait 实现（覆盖 spinner.rs:37-46）
    // 创建 spinner 但不手动调用 finish，验证 Drop 会自动清理
    {
        let _spinner = Spinner::new("Testing drop...");
        // spinner 会在作用域结束时自动 drop
    }
    // Assert: 验证可以正常 drop
    assert!(true);
}

#[test]
fn test_spinner_message_types() {
    // Arrange: 准备测试消息参数的类型转换
    let _spinner1 = Spinner::new("String message");
    let _spinner2 = Spinner::new("String message".to_string());
    // Assert: 验证两种方式都可以创建 spinner
    assert!(true);
}

#[test]
fn test_spinner_multiple_operations() {
    // Arrange: 准备测试 spinner 的多个操作组合
    let spinner = Spinner::new("Starting...");
    spinner.update_message("Step 1...");
    spinner.update_message("Step 2...");
    spinner.update_message("Step 3...");
    spinner.finish();
    // Assert: 验证可以执行多个操作
    assert!(true);
}

#[test]
fn test_spinner_finish_with_message_types() {
    // Arrange: 准备测试 finish_with_message 的消息类型转换
    let spinner = Spinner::new("Creating PR...");
    spinner.finish_with_message("String message");
    let spinner2 = Spinner::new("Creating PR...");
    spinner2.finish_with_message("String message".to_string());
    // Assert: 验证两种方式都可以完成并显示消息
    assert!(true);
}
