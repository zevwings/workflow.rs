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

/// 测试从字符串解析 HTTP 方法（GET）
///
/// ## 测试目的
/// 验证 HttpMethod::from_str() 能够解析 "GET" 字符串。
///
/// ## 测试场景
/// 1. 使用 "GET" 字符串解析 HTTP 方法
/// 2. 验证解析为 Get 方法
///
/// ## 预期结果
/// - "GET" 解析为 HttpMethod::Get
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

/// 测试从字符串解析 HTTP 方法（POST）
///
/// ## 测试目的
/// 验证 HttpMethod::from_str() 能够解析 "POST" 字符串。
///
/// ## 测试场景
/// 1. 使用 "POST" 字符串解析 HTTP 方法
/// 2. 验证解析为 Post 方法
///
/// ## 预期结果
/// - "POST" 解析为 HttpMethod::Post
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

/// 测试从字符串解析 HTTP 方法（PUT）
///
/// ## 测试目的
/// 验证 HttpMethod::from_str() 能够解析 "PUT" 字符串。
///
/// ## 测试场景
/// 1. 使用 "PUT" 字符串解析 HTTP 方法
/// 2. 验证解析为 Put 方法
///
/// ## 预期结果
/// - "PUT" 解析为 HttpMethod::Put
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

/// 测试从字符串解析 HTTP 方法（DELETE）
///
/// ## 测试目的
/// 验证 HttpMethod::from_str() 能够解析 "DELETE" 字符串。
///
/// ## 测试场景
/// 1. 使用 "DELETE" 字符串解析 HTTP 方法
/// 2. 验证解析为 Delete 方法
///
/// ## 预期结果
/// - "DELETE" 解析为 HttpMethod::Delete
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

/// 测试从字符串解析 HTTP 方法（PATCH）
///
/// ## 测试目的
/// 验证 HttpMethod::from_str() 能够解析 "PATCH" 字符串。
///
/// ## 测试场景
/// 1. 使用 "PATCH" 字符串解析 HTTP 方法
/// 2. 验证解析为 Patch 方法
///
/// ## 预期结果
/// - "PATCH" 解析为 HttpMethod::Patch
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

/// 测试从无效字符串解析 HTTP 方法
///
/// ## 测试目的
/// 验证 HttpMethod::from_str() 对无效字符串返回错误。
///
/// ## 测试场景
/// 1. 使用无效的 HTTP 方法字符串解析
/// 2. 验证返回错误且错误消息包含 "Invalid HTTP method"
///
/// ## 预期结果
/// - 无效输入返回错误，错误消息包含 "Invalid HTTP method"
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

/// 测试 HTTP 方法解析大小写敏感性
///
/// ## 测试目的
/// 验证 HttpMethod::from_str() 是大小写敏感的，小写输入返回错误。
///
/// ## 测试场景
/// 1. 使用小写和混合大小写的字符串解析
/// 2. 验证返回错误
///
/// ## 预期结果
/// - 小写和混合大小写输入返回错误
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

/// 测试 HTTP 方法显示格式（GET）
///
/// ## 测试目的
/// 验证 HttpMethod::Get 的 Display trait 实现返回 "GET"。
///
/// ## 测试场景
/// 1. 格式化 Get 方法为字符串
/// 2. 验证显示为 "GET"
///
/// ## 预期结果
/// - Get 方法显示为 "GET"
#[test]
fn test_http_method_display_get_with_get_method_returns_get_string() {
    // Arrange: 准备Get方法

    // Act: 格式化显示
    let display = format!("{}", HttpMethod::Get);

    // Assert: 验证显示为"GET"
    assert_eq!(display, "GET");
}

/// 测试 HTTP 方法显示格式（POST）
///
/// ## 测试目的
/// 验证 HttpMethod::Post 的 Display trait 实现返回 "POST"。
///
/// ## 测试场景
/// 1. 格式化 Post 方法为字符串
/// 2. 验证显示为 "POST"
///
/// ## 预期结果
/// - Post 方法显示为 "POST"
#[test]
fn test_http_method_display_post_with_post_method_returns_post_string() {
    // Arrange: 准备Post方法

    // Act: 格式化显示
    let display = format!("{}", HttpMethod::Post);

    // Assert: 验证显示为"POST"
    assert_eq!(display, "POST");
}

/// 测试 HTTP 方法显示格式（PUT）
///
/// ## 测试目的
/// 验证 HttpMethod::Put 的 Display trait 实现返回 "PUT"。
///
/// ## 测试场景
/// 1. 格式化 Put 方法为字符串
/// 2. 验证显示为 "PUT"
///
/// ## 预期结果
/// - Put 方法显示为 "PUT"
#[test]
fn test_http_method_display_put_with_put_method_returns_put_string() {
    // Arrange: 准备Put方法

    // Act: 格式化显示
    let display = format!("{}", HttpMethod::Put);

    // Assert: 验证显示为"PUT"
    assert_eq!(display, "PUT");
}

/// 测试 HTTP 方法显示格式（DELETE）
///
/// ## 测试目的
/// 验证 HttpMethod::Delete 的 Display trait 实现返回 "DELETE"。
///
/// ## 测试场景
/// 1. 格式化 Delete 方法为字符串
/// 2. 验证显示为 "DELETE"
///
/// ## 预期结果
/// - Delete 方法显示为 "DELETE"
#[test]
fn test_http_method_display_delete_with_delete_method_returns_delete_string() {
    // Arrange: 准备Delete方法

    // Act: 格式化显示
    let display = format!("{}", HttpMethod::Delete);

    // Assert: 验证显示为"DELETE"
    assert_eq!(display, "DELETE");
}

/// 测试 HTTP 方法显示格式（PATCH）
///
/// ## 测试目的
/// 验证 HttpMethod::Patch 的 Display trait 实现返回 "PATCH"。
///
/// ## 测试场景
/// 1. 格式化 Patch 方法为字符串
/// 2. 验证显示为 "PATCH"
///
/// ## 预期结果
/// - Patch 方法显示为 "PATCH"
#[test]
fn test_http_method_display_patch_with_patch_method_returns_patch_string() {
    // Arrange: 准备Patch方法

    // Act: 格式化显示
    let display = format!("{}", HttpMethod::Patch);

    // Assert: 验证显示为"PATCH"
    assert_eq!(display, "PATCH");
}

/// 测试 HTTP 方法 Debug 格式化
///
/// ## 测试目的
/// 验证 HttpMethod 的 Debug trait 实现正确。
///
/// ## 测试场景
/// 1. 格式化 HttpMethod 为 Debug 字符串
/// 2. 验证输出包含方法名称
///
/// ## 预期结果
/// - Debug 输出包含 "Get"
#[test]
fn test_http_method_debug() {
    let debug_str = format!("{:?}", HttpMethod::Get);
    assert!(debug_str.contains("Get"));
}

/// 测试 HTTP 方法克隆功能
///
/// ## 测试目的
/// 验证 HttpMethod 的 Clone trait 实现正确。
///
/// ## 测试场景
/// 1. 创建 HttpMethod 实例
/// 2. 克隆方法
/// 3. 验证克隆后的值与原值相等
///
/// ## 预期结果
/// - 克隆后的值与原值相等
#[test]
fn test_http_method_clone() {
    let original = HttpMethod::Post;
    let cloned = original.clone();
    assert_eq!(format!("{}", original), format!("{}", cloned));
}

/// 测试 HTTP 方法复制功能
///
/// ## 测试目的
/// 验证 HttpMethod 的 Copy trait 实现正确。
///
/// ## 测试场景
/// 1. 创建 HttpMethod 实例
/// 2. 复制方法（通过赋值）
/// 3. 验证复制后的值与原值相等
///
/// ## 预期结果
/// - 复制后的值与原值相等
#[test]
fn test_http_method_copy() {
    let original = HttpMethod::Put;
    let copied = original;
    assert_eq!(format!("{}", original), format!("{}", copied));
}
