//! Git 配置管理测试
//!
//! 测试 Git 配置相关的功能，包括：
//! - 设置全局配置
//! - 读取配置
//! - 配置验证

use pretty_assertions::assert_eq;
use workflow::git::GitConfig;

// ==================== 设置全局用户配置测试 ====================

#[test]
fn test_set_global_user() {
    // 测试设置全局用户配置
    // 注意：这个测试会修改全局 Git 配置，所以需要谨慎
    // 在实际测试中，可能需要先保存原始配置，然后恢复

    let test_email = "test@example.com";
    let test_name = "Test User";

    // 设置全局用户配置
    let result = GitConfig::set_global_user(test_email, test_name).unwrap();

    assert_eq!(result.email, test_email);
    assert_eq!(result.name, test_name);

    // 验证配置已设置
    let (email, name) = GitConfig::get_global_user().unwrap();
    assert_eq!(email, Some(test_email.to_string()));
    assert_eq!(name, Some(test_name.to_string()));
}

// ==================== 读取全局用户配置测试 ====================

#[test]
fn test_get_global_user() {
    // 测试读取全局用户配置
    let (email, name) = GitConfig::get_global_user().unwrap();

    // 配置可能存在也可能不存在，所以只验证返回类型
    // 如果配置存在，应该不为空
    if let Some(ref e) = email {
        assert!(!e.is_empty());
    }
    if let Some(ref n) = name {
        assert!(!n.is_empty());
    }
}

#[test]
fn test_get_global_user_after_set() {
    // 测试设置后读取配置
    let test_email = "test-read@example.com";
    let test_name = "Test Read User";

    // 设置配置
    GitConfig::set_global_user(test_email, test_name).unwrap();

    // 读取配置
    let (email, name) = GitConfig::get_global_user().unwrap();

    assert_eq!(email, Some(test_email.to_string()));
    assert_eq!(name, Some(test_name.to_string()));
}

// ==================== 配置结果测试 ====================

#[test]
fn test_git_config_result() {
    // 测试 GitConfigResult 结构（通过 set_global_user 返回）
    let result = GitConfig::set_global_user("test-result@example.com", "Test Result User").unwrap();

    assert_eq!(result.email, "test-result@example.com");
    assert_eq!(result.name, "Test Result User");
}
