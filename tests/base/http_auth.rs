//! Base HTTP Auth 模块测试
//!
//! 测试 Basic Authentication 的核心功能，包括 Authorization 结构体。

use pretty_assertions::assert_eq;
use workflow::base::http::Authorization;

#[test]
fn test_authorization_new() {
    let auth = Authorization::new("user@example.com", "api_token");
    assert_eq!(auth.username, "user@example.com");
    assert_eq!(auth.password, "api_token");
}

#[test]
fn test_authorization_new_string() {
    let username = String::from("test@example.com");
    let password = String::from("secret_token");
    let auth = Authorization::new(username.clone(), password.clone());
    assert_eq!(auth.username, username);
    assert_eq!(auth.password, password);
}

#[test]
fn test_authorization_new_empty() {
    let auth = Authorization::new("", "");
    assert_eq!(auth.username, "");
    assert_eq!(auth.password, "");
}

#[test]
fn test_authorization_debug() {
    let auth = Authorization::new("user@example.com", "token");
    let debug_str = format!("{:?}", auth);
    assert!(debug_str.contains("user@example.com") || debug_str.contains("token"));
}

#[test]
fn test_authorization_clone() {
    let original = Authorization::new("user@example.com", "token");
    let cloned = original.clone();
    assert_eq!(original.username, cloned.username);
    assert_eq!(original.password, cloned.password);
}

#[test]
fn test_authorization_fields_public() {
    let mut auth = Authorization::new("user@example.com", "token");
    // 验证字段是公开的，可以访问和修改
    assert_eq!(auth.username, "user@example.com");
    assert_eq!(auth.password, "token");

    auth.username = "new@example.com".to_string();
    auth.password = "new_token".to_string();
    assert_eq!(auth.username, "new@example.com");
    assert_eq!(auth.password, "new_token");
}

