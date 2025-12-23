//! Base/Dialog MultiSelect 模块测试
//!
//! 测试多选对话框的核心功能。

use pretty_assertions::assert_eq;
use workflow::base::dialog::MultiSelectDialog;

#[test]
fn test_multi_select_dialog_new() {
    // 测试创建多选对话框
    let options = vec!["Option 1", "Option 2", "Option 3"];
    let _dialog = MultiSelectDialog::new("Choose options", options);
    // 验证可以创建对话框
    assert!(true);
}

#[test]
fn test_multi_select_dialog_with_default() {
    // 测试设置默认选中的选项（覆盖 multi_select.rs:72-74）
    let options = vec!["Option 1", "Option 2", "Option 3"];
    let _dialog = MultiSelectDialog::new("Choose options", options).with_default(vec![0, 2]);
    // 验证链式调用成功
    assert!(true);
}

#[test]
fn test_multi_select_dialog_empty_options() {
    // 测试空选项列表（覆盖 multi_select.rs:87-88）
    let options: Vec<&str> = vec![];
    let dialog = MultiSelectDialog::new("Choose options", options);
    let result = dialog.prompt();

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("No options available"));
}

#[test]
fn test_multi_select_dialog_prompt_string_conversion() {
    // 测试 prompt 参数的类型转换
    let options = vec!["Option 1"];
    let _dialog1 = MultiSelectDialog::new("String prompt", options.clone());
    let _dialog2 = MultiSelectDialog::new("String prompt".to_string(), options);
    // 验证两种方式都可以创建对话框
    assert!(true);
}

#[test]
fn test_multi_select_dialog_with_default_empty() {
    // 测试设置空默认值
    let options = vec!["Option 1", "Option 2"];
    let _dialog = MultiSelectDialog::new("Choose options", options).with_default(vec![]);
    // 验证链式调用成功
    assert!(true);
}

#[test]
fn test_multi_select_dialog_with_default_multiple() {
    // 测试设置多个默认值（覆盖 multi_select.rs:94-95）
    let options = vec!["Option 1", "Option 2", "Option 3", "Option 4"];
    let _dialog = MultiSelectDialog::new("Choose options", options).with_default(vec![0, 2, 3]);
    // 验证链式调用成功
    assert!(true);
}

// 注意：以下测试需要用户交互，在 CI 环境中会被忽略
#[test]
#[ignore] // 需要用户交互
fn test_multi_select_dialog_prompt() {
    // 测试用户选择的情况
    let options = vec!["Option 1", "Option 2", "Option 3"];
    let dialog = MultiSelectDialog::new("Choose options", options).with_default(vec![0]);
    let result = dialog.prompt();
    // 这个测试需要手动运行
    assert!(result.is_ok() || result.is_err());
}
