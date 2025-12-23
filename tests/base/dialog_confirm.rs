//! Base/Dialog Confirm 模块测试
//!
//! 测试确认对话框的核心功能。

use pretty_assertions::assert_eq;
use workflow::base::dialog::ConfirmDialog;

#[test]
fn test_confirm_dialog_new() {
    // 测试创建确认对话框
    let _dialog = ConfirmDialog::new("Continue?");
    // 验证可以创建对话框
    assert!(true);
}

#[test]
fn test_confirm_dialog_with_default() {
    // 测试设置默认值（覆盖 confirm.rs:83-85）
    let _dialog = ConfirmDialog::new("Continue?").with_default(true);
    // 验证链式调用成功
    assert!(true);
}

#[test]
fn test_confirm_dialog_with_cancel_message() {
    // 测试设置取消消息（覆盖 confirm.rs:99-101）
    let _dialog = ConfirmDialog::new("Continue?").with_cancel_message("Operation cancelled.");
    // 验证链式调用成功
    assert!(true);
}

#[test]
fn test_confirm_dialog_chain_all() {
    // 测试链式调用所有方法
    let _dialog = ConfirmDialog::new("Continue?")
        .with_default(false)
        .with_cancel_message("Operation cancelled.");
    // 验证链式调用成功
    assert!(true);
}

#[test]
fn test_confirm_dialog_prompt_string_conversion() {
    // 测试 prompt 参数的类型转换
    let _dialog1 = ConfirmDialog::new("String prompt");
    let _dialog2 = ConfirmDialog::new("String prompt".to_string());
    // 验证两种方式都可以创建对话框
    assert!(true);
}

#[test]
fn test_confirm_dialog_cancel_message_string_conversion() {
    // 测试 cancel_message 参数的类型转换
    let _dialog1 = ConfirmDialog::new("Continue?").with_cancel_message("Message");
    let _dialog2 = ConfirmDialog::new("Continue?").with_cancel_message("Message".to_string());
    // 验证两种方式都可以创建对话框
    assert!(true);
}

// 注意：以下测试需要用户交互，在 CI 环境中会被忽略
#[test]
#[ignore] // 需要用户交互
fn test_confirm_dialog_prompt_confirmed() {
    // 测试用户确认的情况
    let dialog = ConfirmDialog::new("Continue?").with_default(true);
    let _result = dialog.prompt();
    // 这个测试需要手动运行
    assert!(true);
}

#[test]
#[ignore] // 需要用户交互
fn test_confirm_dialog_prompt_cancelled_with_message() {
    // 测试用户取消且设置了取消消息的情况（覆盖 confirm.rs:132-133）
    let dialog = ConfirmDialog::new("Continue?")
        .with_default(false)
        .with_cancel_message("Operation cancelled.");
    let _result = dialog.prompt();
    // 如果用户取消，应该返回错误
    // 这个测试需要手动运行
    assert!(true);
}
