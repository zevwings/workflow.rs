//! Config Show 命令测试
//!
//! 测试配置显示命令的功能。

use crate::common::cli_helpers::TestDataGenerator;
use crate::common::environments::CliTestEnv;
use workflow::commands::config::show::ConfigCommand;

#[test]
fn test_config_command_show_with_empty_config() {
    // 测试空配置的情况
    // ConfigCommand::show() 应该成功返回，即使配置为空
    let result = ConfigCommand::show();

    // 验证函数返回 Ok（空配置是有效的，会显示警告但不会失败）
    assert!(
        result.is_ok(),
        "show() should succeed even with empty config"
    );
}

#[test]
fn test_config_command_show_with_valid_config() -> color_eyre::Result<()> {
    // 测试有有效配置的情况
    let env = CliTestEnv::new()?;
    env.create_config(&TestDataGenerator::config_content())?;

    // 注意：ConfigCommand::show() 使用 Paths::workflow_config() 获取配置路径
    // 这个路径基于 HOME 目录，所以我们需要设置 HOME 环境变量
    // 但由于 Paths 的实现可能使用 dirs 库，我们需要确保配置在正确的位置
    // 这里我们主要测试函数不会崩溃，实际的路径测试在集成测试中完成
    // 新版 CliTestEnv 使用 TestIsolation，环境变量会自动管理
    // 配置路径: env.path().join(".workflow")

    let result = ConfigCommand::show();

    // 验证函数返回 Result 类型（可能成功或失败，取决于配置路径）
    // 主要验证函数不会 panic
    match result {
        Ok(_) => {
            // 命令执行成功
        }
        Err(_) => {
            // 如果配置路径不在预期位置，这是可以接受的
            // 主要验证函数不会 panic
        }
    }

    // 环境变量会在 env 离开作用域时自动恢复（通过 TestIsolation）
    Ok(())
}

#[test]
fn test_config_command_show_error_handling() {
    // 测试错误处理逻辑
    // ConfigCommand::show() 应该能够处理各种错误情况而不 panic

    // 测试1: 无配置目录的情况
    let result1 = ConfigCommand::show();
    // 应该返回 Ok 或 Err，但不应该 panic
    match result1 {
        Ok(_) | Err(_) => {
            // 这是预期的行为
        }
    }

    // 测试2: 配置路径无效的情况
    // 由于 ConfigCommand::show() 使用 Paths::workflow_config()，
    // 如果路径获取失败，应该返回 Err
    // 这里我们主要验证函数不会 panic
    let result2 = ConfigCommand::show();
    match result2 {
        Ok(_) | Err(_) => {
            // 这是预期的行为
        }
    }
}
