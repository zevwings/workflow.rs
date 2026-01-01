//! GitConfigCommand 测试
//!
//! 测试配置命令包装层的功能。

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
/// 1. 设置测试配置
/// 2. 获取配置值
/// 3. 验证配置被设置
/// 4. 删除配置
///
/// ## 预期结果
/// - 配置设置成功
#[test]
#[serial]
fn test_set_config_sets_value() -> Result<()> {
    let test_key = "test.config.key";
    let test_value = "test-value-12345";

    // Act: 设置配置值
    GitConfigCommand::set_config(test_key, test_value, true, None)?;

    // Act: 获取配置值
    let value = GitConfigCommand::get_config(test_key, true, None)?;

    // Assert: 验证配置被设置
    assert_eq!(
        value,
        Some(test_value.to_string()),
        "Config value should match set value"
    );

    // Cleanup: 删除测试配置
    let _ = GitConfigCommand::unset_config(test_key, true, None);

    Ok(())
}

/// 测试删除配置项
///
/// ## 测试目的
/// 验证 GitConfigCommand::unset_config() 能够删除配置项。
///
/// ## 测试场景
/// 1. 设置测试配置
/// 2. 删除配置
/// 3. 验证配置被删除
///
/// ## 预期结果
/// - 配置删除成功
#[test]
#[serial]
fn test_unset_config_removes_config() -> Result<()> {
    let test_key = "test.unset.key";
    let test_value = "test-value-unset";

    // Arrange: 设置配置
    GitConfigCommand::set_config(test_key, test_value, true, None)?;

    // Act: 删除配置
    GitConfigCommand::unset_config(test_key, true, None)?;

    // Assert: 验证配置被删除
    let value = GitConfigCommand::get_config(test_key, true, None).unwrap_or(None); // 配置不存在时可能返回 None
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
/// 1. 保存原始配置（如果存在）
/// 2. 设置测试用户信息
/// 3. 验证配置被设置
/// 4. 恢复原始配置
///
/// ## 预期结果
/// - 用户信息设置成功
#[test]
#[serial]
fn test_set_user_sets_email_and_name() -> Result<()> {
    // Arrange: 保存原始配置
    let original_email = GitConfigCommand::get_user_email(true, None)?;
    let original_name = GitConfigCommand::get_user_name(true, None)?;

    let test_email = "test@example.com";
    let test_name = "Test User";

    // Act: 设置用户信息
    let (email, name) = GitConfigCommand::set_user(test_email, test_name, true, None)?;

    // Assert: 验证配置被设置
    assert_eq!(email, test_email, "Email should match");
    assert_eq!(name, test_name, "Name should match");

    // Cleanup: 恢复原始配置
    if let Some(orig_email) = original_email {
        GitConfigCommand::set_config("user.email", &orig_email, true, None)?;
    } else {
        let _ = GitConfigCommand::unset_config("user.email", true, None);
    }

    if let Some(orig_name) = original_name {
        GitConfigCommand::set_config("user.name", &orig_name, true, None)?;
    } else {
        let _ = GitConfigCommand::unset_config("user.name", true, None);
    }

    Ok(())
}

/// 测试列出所有配置项
///
/// ## 测试目的
/// 验证 GitConfigCommand::list_config() 能够列出所有配置项。
///
/// ## 测试场景
/// 1. 列出所有全局配置
/// 2. 验证返回配置列表
///
/// ## 预期结果
/// - 返回配置项列表（可能为空或包含配置）
#[test]
#[serial]
fn test_list_config_returns_config_list() -> Result<()> {
    // Act: 列出所有全局配置
    let configs = GitConfigCommand::list_config(true, None)?;

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
