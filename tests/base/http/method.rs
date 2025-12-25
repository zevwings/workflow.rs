//! Base HTTP Method 模块测试
//!
//! 测试 HTTP 方法枚举的核心功能，包括 FromStr 和 Display trait。
//!
//! ## 测试策略
//!
//! - 使用 `expect()` 替代 `unwrap()` 提供清晰的错误消息
//! - 测试所有HTTP方法的解析和显示功能

use pretty_assertions::assert_eq;
use std::str::FromStr;
use workflow::base::http::HttpMethod;

// ==================== HttpMethod Parsing Tests ====================

#[test]
fn test_http_method_from_str_get_with_get_string_returns_get() {
    // Arrange: 准备GET字符串

    // Act: 从字符串解析HTTP方法
    let method = HttpMethod::from_str("GET").expect("GET should be a valid HTTP method");

    // Assert: 验证解析为Get方法
    match method {
        HttpMethod::Get => assert!(true),
        _ => panic!("Expected Get"),
    }
}

#[test]
fn test_http_method_from_str_post_with_post_string_returns_post() {
    // Arrange: 准备POST字符串

    // Act: 从字符串解析HTTP方法
    let method = HttpMethod::from_str("POST").expect("POST should be a valid HTTP method");

    // Assert: 验证解析为Post方法
    match method {
        HttpMethod::Post => assert!(true),
        _ => panic!("Expected Post"),
    }
}

#[test]
fn test_http_method_from_str_put_with_put_string_returns_put() {
    // Arrange: 准备PUT字符串

    // Act: 从字符串解析HTTP方法
    let method = HttpMethod::from_str("PUT").expect("PUT should be a valid HTTP method");

    // Assert: 验证解析为Put方法
    match method {
        HttpMethod::Put => assert!(true),
        _ => panic!("Expected Put"),
    }
}

#[test]
fn test_http_method_from_str_delete_with_delete_string_returns_delete() {
    // Arrange: 准备DELETE字符串

    // Act: 从字符串解析HTTP方法
    let method = HttpMethod::from_str("DELETE").expect("DELETE should be a valid HTTP method");

    // Assert: 验证解析为Delete方法
    match method {
        HttpMethod::Delete => assert!(true),
        _ => panic!("Expected Delete"),
    }
}

#[test]
fn test_http_method_from_str_patch_with_patch_string_returns_patch() {
    // Arrange: 准备PATCH字符串

    // Act: 从字符串解析HTTP方法
    let method = HttpMethod::from_str("PATCH").expect("PATCH should be a valid HTTP method");

    // Assert: 验证解析为Patch方法
    match method {
        HttpMethod::Patch => assert!(true),
        _ => panic!("Expected Patch"),
    }
}

#[test]
fn test_http_method_from_str_invalid_with_invalid_string_returns_error() {
    // Arrange: 准备无效的HTTP方法字符串
    let invalid_method = "INVALID";

    // Act: 尝试解析无效方法
    let result = HttpMethod::from_str(invalid_method);

    // Assert: 验证返回错误且错误消息包含"Invalid HTTP method"
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid HTTP method"));
}

#[test]
fn test_http_method_from_str_case_sensitive_with_lowercase_strings_returns_error() {
    // Arrange: 准备小写和混合大小写的字符串
    let lowercase_inputs = ["get", "post", "Get"];

    // Act & Assert: 验证大小写敏感，小写输入返回错误
    for input in lowercase_inputs.iter() {
        assert!(HttpMethod::from_str(input).is_err(), "Input '{}' should fail", input);
    }
}

// ==================== HttpMethod Display Tests ====================

#[test]
fn test_http_method_display_get_with_get_method_returns_get_string() {
    // Arrange: 准备Get方法

    // Act: 格式化显示
    let display = format!("{}", HttpMethod::Get);

    // Assert: 验证显示为"GET"
    assert_eq!(display, "GET");
}

#[test]
fn test_http_method_display_post_with_post_method_returns_post_string() {
    // Arrange: 准备Post方法

    // Act: 格式化显示
    let display = format!("{}", HttpMethod::Post);

    // Assert: 验证显示为"POST"
    assert_eq!(display, "POST");
}

#[test]
fn test_http_method_display_put_with_put_method_returns_put_string() {
    // Arrange: 准备Put方法

    // Act: 格式化显示
    let display = format!("{}", HttpMethod::Put);

    // Assert: 验证显示为"PUT"
    assert_eq!(display, "PUT");
}

#[test]
fn test_http_method_display_delete_with_delete_method_returns_delete_string() {
    // Arrange: 准备Delete方法

    // Act: 格式化显示
    let display = format!("{}", HttpMethod::Delete);

    // Assert: 验证显示为"DELETE"
    assert_eq!(display, "DELETE");
}

#[test]
fn test_http_method_display_patch_with_patch_method_returns_patch_string() {
    // Arrange: 准备Patch方法

    // Act: 格式化显示
    let display = format!("{}", HttpMethod::Patch);

    // Assert: 验证显示为"PATCH"
    assert_eq!(display, "PATCH");
}

#[test]
fn test_http_method_debug() {
    let debug_str = format!("{:?}", HttpMethod::Get);
    assert!(debug_str.contains("Get"));
}

#[test]
fn test_http_method_clone() {
    let original = HttpMethod::Post;
    let cloned = original.clone();
    assert_eq!(format!("{}", original), format!("{}", cloned));
}

#[test]
fn test_http_method_copy() {
    let original = HttpMethod::Put;
    let copied = original;
    assert_eq!(format!("{}", original), format!("{}", copied));
}
