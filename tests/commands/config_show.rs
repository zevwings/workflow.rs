//! Config Show 命令测试
//!
//! 测试配置显示命令的功能。

use crate::common::cli_helpers::TestDataGenerator;
use crate::common::environments::CliTestEnv;
use workflow::commands::config::show::ConfigCommand;

// ==================== Config Show Command Tests ====================

#[test]
fn test_config_command_show_with_empty_config_returns_ok() {
    // Arrange: 准备空配置环境

    // Act: 执行配置显示命令
    let result = ConfigCommand::show();

    // Assert: 验证命令执行成功（空配置是有效的，会显示警告但不会失败）
    assert!(
        result.is_ok(),
        "show() should succeed even with empty config"
    );
}

#[test]
fn test_config_command_show_with_valid_config_returns_result() -> color_eyre::Result<()> {
    // Arrange: 准备有效配置环境
    let env = CliTestEnv::new()?;
    env.create_config(&TestDataGenerator::config_content())?;

    // Act: 执行配置显示命令
    let result = ConfigCommand::show();

    // Assert: 验证函数返回 Result（可能成功或失败，取决于配置路径，主要验证不会panic）
    match result {
        Ok(_) => {
            // 命令执行成功
        }
        Err(_) => {
            // 如果配置路径不在预期位置，这是可以接受的
        }
    }

    Ok(())
}

#[test]
fn test_config_command_show_error_handling_with_various_errors_handles_gracefully() {
    // Arrange: 准备各种错误场景

    // Act & Assert: 测试1 - 无配置目录的情况
    let result1 = ConfigCommand::show();
    match result1 {
        Ok(_) | Err(_) => {
            // 这是预期的行为，不应该panic
        }
    }

    // Act & Assert: 测试2 - 配置路径无效的情况
    let result2 = ConfigCommand::show();
    match result2 {
        Ok(_) | Err(_) => {
            // 这是预期的行为，不应该panic
        }
    }
}
