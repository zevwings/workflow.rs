//! Git Auth 模块集成测试
//!
//! 测试 Git 远程操作认证相关的公共 API，包括：
//! - find_ssh_key() 方法的基本功能
//! - extract_host_from_url() 方法的基本功能

use workflow::git::GitAuth;

// ==================== SSH Key Tests ====================

/// 测试查找 SSH 密钥（基本功能）
///
/// ## 测试目的
/// 验证 `GitAuth::find_ssh_key()` 方法能够查找 SSH 密钥。
///
/// ## 测试场景
/// 1. 调用 find_ssh_key() 方法
/// 2. 验证方法不会 panic
/// 3. 验证返回结果（可能为 None，如果用户没有配置 SSH 密钥）
///
/// ## 预期结果
/// - 方法成功执行不 panic
/// - 如果用户有 SSH 密钥，应该能找到
/// - 如果没有，返回 None 也是正常的
#[test]
fn test_find_ssh_key() {
    // Act: 查找 SSH 密钥（不应该 panic）
    let _key_path = GitAuth::find_ssh_key();

    // Assert: 如果用户有 SSH 密钥，应该能找到
    // 如果没有，返回 None 也是正常的
}

/// 测试从 URL 提取 host（基本功能）
///
/// ## 测试目的
/// 验证 `GitAuth::extract_host_from_url()` 方法能够从 URL 中提取 host。
///
/// ## 测试场景
/// 1. 测试 SSH URL 格式（git@host:path）
/// 2. 测试 HTTPS URL 格式（https://host/path）
/// 3. 验证提取的 host 正确
///
/// ## 预期结果
/// - 能够正确提取 host
#[test]
fn test_extract_host_from_url() {
    // Act & Assert: 测试 SSH URL 格式
    let host1 = GitAuth::extract_host_from_url("git@github.com:user/repo.git");
    assert_eq!(host1, Some("github.com".to_string()));

    // Act & Assert: 测试 HTTPS URL 格式
    let host2 = GitAuth::extract_host_from_url("https://github.com/user/repo.git");
    assert_eq!(host2, Some("github.com".to_string()));

    // Act & Assert: 测试无效 URL
    let host3 = GitAuth::extract_host_from_url("invalid-url");
    assert_eq!(host3, None);
}
