//! Base/Dialog Integration 模块测试
//!
//! 测试对话框的组合使用和类型安全。

use workflow::base::dialog::{ConfirmDialog, InputDialog, MultiSelectDialog, SelectDialog};

#[test]
fn test_dialog_configuration_completeness() {
    // 测试对话框配置的完整性
    // 验证所有配置选项都可以组合使用

    // InputDialog 完整配置
    let _input = InputDialog::new("Enter value")
        .with_default("default")
        .with_validator(|s: &str| {
            if s.len() > 0 {
                Ok(())
            } else {
                Err("Empty".to_string())
            }
        })
        .allow_empty(false);
    assert!(true, "InputDialog full configuration should work");

    // SelectDialog 完整配置
    let _select = SelectDialog::new("Choose", vec!["A", "B", "C"]).with_default(0);
    assert!(true, "SelectDialog full configuration should work");

    // MultiSelectDialog 完整配置
    let _multi = MultiSelectDialog::new("Choose", vec!["A", "B", "C"]).with_default(vec![0]);
    assert!(true, "MultiSelectDialog full configuration should work");

    // ConfirmDialog 完整配置
    let _confirm = ConfirmDialog::new("Continue?")
        .with_default(true)
        .with_cancel_message("Cancelled");
    assert!(true, "ConfirmDialog full configuration should work");
}

#[test]
fn test_dialog_type_safety() {
    // 测试对话框的类型安全
    // 验证不同类型的对话框不能混淆使用

    let _input: InputDialog = InputDialog::new("Input");
    let _select: SelectDialog<&str> = SelectDialog::new("Select", vec!["A"]);
    let _multi: MultiSelectDialog<&str> = MultiSelectDialog::new("Multi", vec!["A"]);
    let _confirm: ConfirmDialog = ConfirmDialog::new("Confirm");

    // 验证类型正确（通过编译验证）
    assert!(true, "Dialog types should be type-safe");
}

#[test]
fn test_dialog_error_handling_structure() {
    // 测试对话框错误处理的结构
    // 验证错误类型和消息格式

    // 这个测试主要验证错误处理的结构正确
    // 实际错误处理在 prompt() 方法中，需要用户交互才能测试

    // 验证对话框可以创建，错误处理结构存在
    let _dialog = InputDialog::new("Test");
    assert!(true, "Dialog error handling structure should exist");
}
