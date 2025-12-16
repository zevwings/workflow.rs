//! HTTP 认证测试
//!
//! 测试 HTTP Basic Authentication 功能，包括：
//! - Authorization 结构体测试
//! - 认证信息创建测试
//! - 边界条件测试

use workflow::base::http::auth::Authorization;

// ==================== Authorization 创建测试 ====================

#[test]
fn test_authorization_new() {
    let auth = Authorization::new("user@example.com", "token123");
    assert_eq!(auth.username, "user@example.com");
    assert_eq!(auth.password, "token123");
}

#[test]
fn test_authorization_new_with_string() {
    let username = String::from("user@example.com");
    let password = String::from("token123");
    let auth = Authorization::new(username.clone(), password.clone());
    assert_eq!(auth.username, username);
    assert_eq!(auth.password, password);
}

#[test]
fn test_authorization_new_with_str() {
    let auth = Authorization::new("user", "pass");
    assert_eq!(auth.username, "user");
    assert_eq!(auth.password, "pass");
}

// ==================== 边界条件测试 ====================

#[test]
fn test_authorization_empty_username() {
    let auth = Authorization::new("", "token123");
    assert_eq!(auth.username, "");
    assert_eq!(auth.password, "token123");
}

#[test]
fn test_authorization_empty_password() {
    let auth = Authorization::new("user@example.com", "");
    assert_eq!(auth.username, "user@example.com");
    assert_eq!(auth.password, "");
}

#[test]
fn test_authorization_empty_both() {
    let auth = Authorization::new("", "");
    assert_eq!(auth.username, "");
    assert_eq!(auth.password, "");
}

#[test]
fn test_authorization_special_characters() {
    let auth = Authorization::new("user@example.com", "token!@#$%^&*()");
    assert_eq!(auth.username, "user@example.com");
    assert_eq!(auth.password, "token!@#$%^&*()");
}

#[test]
fn test_authorization_unicode() {
    let auth = Authorization::new("用户@example.com", "密码123");
    assert_eq!(auth.username, "用户@example.com");
    assert_eq!(auth.password, "密码123");
}

#[test]
fn test_authorization_long_strings() {
    let long_username = "a".repeat(1000);
    let long_password = "b".repeat(1000);
    let auth = Authorization::new(&long_username, &long_password);
    assert_eq!(auth.username, long_username);
    assert_eq!(auth.password, long_password);
}

// ==================== Clone 测试 ====================

#[test]
fn test_authorization_clone() {
    let auth1 = Authorization::new("user@example.com", "token123");
    let auth2 = auth1.clone();
    assert_eq!(auth1.username, auth2.username);
    assert_eq!(auth1.password, auth2.password);
}

// ==================== Debug 测试 ====================

#[test]
fn test_authorization_debug() {
    let auth = Authorization::new("user@example.com", "token123");
    let debug_str = format!("{:?}", auth);
    assert!(debug_str.contains("user@example.com"));
    // 注意：密码可能不会在 Debug 输出中显示（安全考虑）
}
