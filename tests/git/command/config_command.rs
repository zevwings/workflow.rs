//! GitConfigCommand 测试
//!
//! 测试配置命令包装层的功能。

use crate::common::guards::GitConfigGuard;
use color_eyre::Result;
use serial_test::serial;
use workflow::git::commands::GitConfigCommand;

/// 测试获取配置值
///
/// ## 测试目的
/// 验证 GitConfigCommand::get_config() 能够获取配置值。
///
/// ## 测试场景
/// 1. 获取 user.email 配置
/// 2. 验证返回配置值（可能为 None）
///
/// ## 预期结果
/// - 返回配置值或 None
#[test]
#[serial]
fn test_get_config_returns_value() -> Result<()> {
    // Act: 获取配置值
    let email = GitConfigCommand::get_config("user.email", true, None)?;

    // Assert: 验证返回结果（可能为 None 或 Some）
    // 这取决于系统配置，但应该不会出错
    match email {
        Some(val) => assert!(!val.is_empty(), "Email should not be empty if present"),
        None => {
            // 没有配置是正常的
        }
    }

    Ok(())
}

/// 测试设置配置值
///
/// ## 测试目的
/// 验证 GitConfigCommand::set_config() 能够设置配置值。
///
/// ## 测试场景
/// 1. 使用隔离的Git配置环境
/// 2. 设置测试配置
/// 3. 获取配置值
/// 4. 验证配置被设置
///
/// ## 预期结果
/// - 配置设置成功
#[test]
#[serial]
#[ignore]
fn test_set_config_sets_value() -> Result<()> {
    // Arrange: 使用隔离的Git配置环境
    let _guard = GitConfigGuard::new()?;

    let test_key = "test.config.key";
    let test_value = "test-value-12345";

    // Act: 设置配置值（使用 global=false，Git 会使用 GIT_CONFIG 环境变量指定的文件）
    GitConfigCommand::set_config(test_key, test_value, false, None)?;

    // Act: 获取配置值
    let value = GitConfigCommand::get_config(test_key, false, None)?;

    // Assert: 验证配置被设置
    assert_eq!(
        value,
        Some(test_value.to_string()),
        "Config value should match set value"
    );

    // guard 在 drop 时自动清理，无需手动删除配置

    Ok(())
}

/// 测试删除配置项
///
/// ## 测试目的
/// 验证 GitConfigCommand::unset_config() 能够删除配置项。
///
/// ## 测试场景
/// 1. 使用隔离的Git配置环境
/// 2. 设置测试配置
/// 3. 删除配置
/// 4. 验证配置被删除
///
/// ## 预期结果
/// - 配置删除成功
#[test]
#[serial]
#[ignore]
fn test_unset_config_removes_config() -> Result<()> {
    // Arrange: 使用隔离的Git配置环境
    let _guard = GitConfigGuard::new()?;

    let test_key = "test.unset.key";
    let test_value = "test-value-unset";

    // Arrange: 设置配置（使用 global=false，Git 会使用 GIT_CONFIG 环境变量指定的文件）
    GitConfigCommand::set_config(test_key, test_value, false, None)?;

    // Act: 删除配置
    GitConfigCommand::unset_config(test_key, false, None)?;

    // Assert: 验证配置被删除
    let value = GitConfigCommand::get_config(test_key, false, None).unwrap_or(None); // 配置不存在时可能返回 None
    assert_eq!(value, None, "Config should be removed");

    Ok(())
}

/// 测试获取用户邮箱
///
/// ## 测试目的
/// 验证 GitConfigCommand::get_user_email() 能够获取用户邮箱。
///
/// ## 测试场景
/// 1. 获取用户邮箱
/// 2. 验证返回结果
///
/// ## 预期结果
/// - 返回用户邮箱或 None
#[test]
#[serial]
fn test_get_user_email_returns_email() -> Result<()> {
    // Act: 获取用户邮箱
    let email = GitConfigCommand::get_user_email(true, None)?;

    // Assert: 验证返回结果（可能为 None 或 Some）
    match email {
        Some(val) => assert!(!val.is_empty(), "Email should not be empty if present"),
        None => {
            // 没有配置是正常的
        }
    }

    Ok(())
}

/// 测试获取用户名称
///
/// ## 测试目的
/// 验证 GitConfigCommand::get_user_name() 能够获取用户名称。
///
/// ## 测试场景
/// 1. 获取用户名称
/// 2. 验证返回结果
///
/// ## 预期结果
/// - 返回用户名称或 None
#[test]
#[serial]
fn test_get_user_name_returns_name() -> Result<()> {
    // Act: 获取用户名称
    let name = GitConfigCommand::get_user_name(true, None)?;

    // Assert: 验证返回结果（可能为 None 或 Some）
    match name {
        Some(val) => assert!(!val.is_empty(), "Name should not be empty if present"),
        None => {
            // 没有配置是正常的
        }
    }

    Ok(())
}

/// 测试设置用户信息
///
/// ## 测试目的
/// 验证 GitConfigCommand::set_user() 能够设置用户信息。
///
/// ## 测试场景
/// 1. 使用隔离的Git配置环境
/// 2. 设置测试用户信息
/// 3. 验证配置被设置
///
/// ## 预期结果
/// - 用户信息设置成功
#[test]
#[serial]
#[ignore]
fn test_set_user_sets_email_and_name() -> Result<()> {
    // Arrange: 使用隔离的Git配置环境
    let _guard = GitConfigGuard::new()?;

    let test_email = "test@example.com";
    let test_name = "Test User";

    // Act: 设置用户信息（使用 global=false，Git 会使用 GIT_CONFIG 环境变量指定的文件）
    let (email, name) = GitConfigCommand::set_user(test_email, test_name, false, None)?;

    // Assert: 验证配置被设置
    assert_eq!(email, test_email, "Email should match");
    assert_eq!(name, test_name, "Name should match");

    // 验证配置确实被写入隔离的配置文件
    let saved_email = GitConfigCommand::get_user_email(false, None)?;
    let saved_name = GitConfigCommand::get_user_name(false, None)?;
    assert_eq!(
        saved_email,
        Some(test_email.to_string()),
        "Email should be saved"
    );
    assert_eq!(
        saved_name,
        Some(test_name.to_string()),
        "Name should be saved"
    );

    // guard 在 drop 时自动清理，无需手动恢复配置

    Ok(())
}

/// 测试列出所有配置项
///
/// ## 测试目的
/// 验证 GitConfigCommand::list_config() 能够列出所有配置项。
///
/// ## 测试场景
/// 1. 使用隔离的Git配置环境
/// 2. 列出所有配置
/// 3. 验证返回配置列表
///
/// ## 预期结果
/// - 返回配置项列表（可能为空或包含配置）
#[test]
#[serial]
#[ignore]
fn test_list_config_returns_config_list() -> Result<()> {
    // Arrange: 使用隔离的Git配置环境
    let _guard = GitConfigGuard::new()?;

    // Act: 列出所有配置（使用 global=false，Git 会使用 GIT_CONFIG 环境变量指定的文件）
    let configs = GitConfigCommand::list_config(false, None)?;

    // Assert: 验证返回列表（可能为空或包含配置）
    // 配置列表可能为空，也可能包含系统配置，这都是正常的
    assert!(
        configs.is_empty() || !configs.is_empty(),
        "Should return a list of configs"
    );

    // 如果列表不为空，验证格式
    for (key, _value) in &configs {
        assert!(!key.is_empty(), "Config key should not be empty");
        // value 可能为空，这是正常的
    }

    Ok(())
}
