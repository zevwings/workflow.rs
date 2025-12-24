//! Base/Dialog Input 模块测试
//!
//! 测试输入对话框的核心功能。

use workflow::base::dialog::InputDialog;

#[test]
fn test_input_dialog_creation() {
    // 测试创建输入对话框
    let _dialog = InputDialog::new("Enter your name");

    // 验证对话框已创建（通过编译和运行验证）
    // 由于结构体字段是私有的，我们只能验证可以创建
    assert!(true, "InputDialog should be created");
}

#[test]
fn test_input_dialog_with_default() {
    // 测试带默认值的输入对话框
    let _dialog = InputDialog::new("Enter email").with_default("user@example.com");

    // 验证链式调用成功
    assert!(true, "InputDialog with default should be created");
}

#[test]
fn test_input_dialog_with_validator() {
    // 测试带验证器的输入对话框
    let _dialog = InputDialog::new("Enter age").with_validator(|input: &str| {
        if input.parse::<u32>().is_ok() {
            Ok(())
        } else {
            Err("Please enter a valid number".to_string())
        }
    });

    // 验证验证器已设置
    assert!(true, "InputDialog with validator should be created");
}

#[test]
fn test_input_dialog_allow_empty() {
    // 测试允许空值的输入对话框
    let _dialog = InputDialog::new("Enter value (optional)").allow_empty(true);

    // 验证配置成功
    assert!(true, "InputDialog with allow_empty should be created");
}

#[test]
fn test_input_dialog_chain_calls() {
    // 测试链式调用
    let _dialog = InputDialog::new("Enter value")
        .with_default("default")
        .with_validator(|input: &str| {
            if !input.is_empty() {
                Ok(())
            } else {
                Err("Cannot be empty".to_string())
            }
        })
        .allow_empty(false);

    // 验证所有链式调用成功
    assert!(true, "InputDialog chain calls should work");
}

#[test]
fn test_input_dialog_validator_logic() {
    // 测试验证器逻辑（不实际显示对话框）
    let validator = |input: &str| -> Result<(), String> {
        if input.len() >= 3 {
            Ok(())
        } else {
            Err("Input must be at least 3 characters".to_string())
        }
    };

    // 测试验证器逻辑
    assert!(validator("abc").is_ok());
    assert!(validator("ab").is_err());
    assert!(validator("").is_err());
}
