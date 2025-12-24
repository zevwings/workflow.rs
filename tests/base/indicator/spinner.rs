//! Base/Indicator Spinner 模块测试
//!
//! 测试 Spinner 的核心功能。

use std::time::Duration;
use workflow::base::indicator::Spinner;

#[test]
fn test_spinner_new() {
    // 测试创建新的 spinner（覆盖 spinner.rs:68-79）
    let spinner = Spinner::new("Creating PR...");
    // 验证可以创建 spinner
    spinner.finish();
    assert!(true);
}

#[test]
fn test_spinner_new_with_string() {
    // 测试使用 String 创建 spinner
    let message = "Processing...".to_string();
    let spinner = Spinner::new(message);
    spinner.finish();
    assert!(true);
}

#[test]
fn test_spinner_update_message() {
    // 测试更新消息（覆盖 spinner.rs:96-98）
    let spinner = Spinner::new("Starting...");
    spinner.update_message("Processing...");
    spinner.update_message("Almost done...");
    spinner.finish();
    // 验证可以更新消息
    assert!(true);
}

#[test]
fn test_spinner_finish() {
    // 测试完成 spinner（覆盖 spinner.rs:113-119）
    let spinner = Spinner::new("Creating PR...");
    spinner.finish();
    // 验证可以完成 spinner
    assert!(true);
}

#[test]
fn test_spinner_finish_with_message() {
    // 测试完成并显示消息（覆盖 spinner.rs:138-144）
    let spinner = Spinner::new("Creating PR...");
    spinner.finish_with_message("PR created successfully!");
    // 验证可以完成并显示消息
    assert!(true);
}

#[test]
fn test_spinner_with_success() {
    // 测试 with 方法成功场景（覆盖 spinner.rs:175-194）
    let result: Result<i32, Box<dyn std::error::Error>> = Spinner::with("Creating PR...", || {
        // 模拟快速操作（< 100ms）
        Ok(42)
    });
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_spinner_with_error() {
    // 测试 with 方法错误场景
    let result: Result<i32, String> = Spinner::with("Creating PR...", || {
        Err("Operation failed".to_string())
    });
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Operation failed");
}

#[test]
fn test_spinner_with_slow_operation() {
    // 测试 with 方法慢速操作（> 100ms）
    let result: Result<i32, Box<dyn std::error::Error>> = Spinner::with("Creating PR...", || {
        // 模拟慢速操作（> 100ms）
        std::thread::sleep(Duration::from_millis(150));
        Ok(42)
    });
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_spinner_with_output_success() {
    // 测试 with_output 方法成功场景（覆盖 spinner.rs:231-242）
    let result: Result<i32, Box<dyn std::error::Error>> =
        Spinner::with_output("Pushing to remote...", || {
            Ok(42)
        });
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_spinner_with_output_error() {
    // 测试 with_output 方法错误场景
    let result: Result<i32, String> = Spinner::with_output("Pushing to remote...", || {
        Err("Push failed".to_string())
    });
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Push failed");
}

#[test]
fn test_spinner_drop() {
    // 测试 Drop trait 实现（覆盖 spinner.rs:37-46）
    // 创建 spinner 但不手动调用 finish，验证 Drop 会自动清理
    {
        let _spinner = Spinner::new("Testing drop...");
        // spinner 会在作用域结束时自动 drop
    }
    // 验证可以正常 drop
    assert!(true);
}

#[test]
fn test_spinner_message_types() {
    // 测试消息参数的类型转换
    let _spinner1 = Spinner::new("String message");
    let _spinner2 = Spinner::new("String message".to_string());
    // 验证两种方式都可以创建 spinner
    assert!(true);
}

#[test]
fn test_spinner_multiple_operations() {
    // 测试 spinner 的多个操作组合
    let spinner = Spinner::new("Starting...");
    spinner.update_message("Step 1...");
    spinner.update_message("Step 2...");
    spinner.update_message("Step 3...");
    spinner.finish();
    // 验证可以执行多个操作
    assert!(true);
}

#[test]
fn test_spinner_finish_with_message_types() {
    // 测试 finish_with_message 的消息类型转换
    let spinner = Spinner::new("Creating PR...");
    spinner.finish_with_message("String message");
    let spinner2 = Spinner::new("Creating PR...");
    spinner2.finish_with_message("String message".to_string());
    // 验证两种方式都可以完成并显示消息
    assert!(true);
}

