//! Base HTTP Auth 模块测试
//!
//! 测试 Basic Authentication 的核心功能，包括 Authorization 结构体。

use pretty_assertions::assert_eq;
use workflow::base::http::Authorization;

// ==================== Authorization Creation Tests ====================

#[test]
fn test_authorization_new_with_valid_credentials_creates_instance() {
    // Arrange: 准备用户名和密码
    let username = "user@example.com";
    let password = "api_token";

    // Act: 创建 Authorization 实例
    let auth = Authorization::new(username, password);

    // Assert: 验证字段值正确
    assert_eq!(auth.username, username);
    assert_eq!(auth.password, password);
}

#[test]
fn test_authorization_new_with_string_credentials_creates_instance() {
    // Arrange: 准备 String 类型的用户名和密码
    let username = String::from("test@example.com");
    let password = String::from("secret_token");

    // Act: 创建 Authorization 实例
    let auth = Authorization::new(username.clone(), password.clone());

    // Assert: 验证字段值正确
    assert_eq!(auth.username, username);
    assert_eq!(auth.password, password);
}

#[test]
fn test_authorization_new_with_empty_credentials_creates_instance() {
    // Arrange: 准备空字符串
    let username = "";
    let password = "";

    // Act: 创建 Authorization 实例
    let auth = Authorization::new(username, password);

    // Assert: 验证字段值为空字符串
    assert_eq!(auth.username, username);
    assert_eq!(auth.password, password);
}

// ==================== Authorization Debug Tests ====================

#[test]
fn test_authorization_debug_with_valid_instance_returns_debug_string() {
    // Arrange: 准备 Authorization 实例
    let auth = Authorization::new("user@example.com", "token");

    // Act: 格式化 Debug 输出
    let debug_str = format!("{:?}", auth);

    // Assert: 验证 Debug 字符串包含用户名或密码
    assert!(debug_str.contains("user@example.com") || debug_str.contains("token"));
}

// ==================== Authorization Clone Tests ====================

#[test]
fn test_authorization_clone_with_valid_instance_creates_clone() {
    // Arrange: 准备原始 Authorization 实例
    let original = Authorization::new("user@example.com", "token");

    // Act: 克隆实例
    let cloned = original.clone();

    // Assert: 验证克隆的字段值与原始值相同
    assert_eq!(original.username, cloned.username);
    assert_eq!(original.password, cloned.password);
}

// ==================== Authorization Field Access Tests ====================

#[test]
fn test_authorization_fields_with_public_access_allows_modification() {
    // Arrange: 准备 Authorization 实例
    let mut auth = Authorization::new("user@example.com", "token");

    // Act: 修改字段值
    auth.username = "new@example.com".to_string();
    auth.password = "new_token".to_string();

    // Assert: 验证字段值已更新
    assert_eq!(auth.username, "new@example.com");
    assert_eq!(auth.password, "new_token");
}
