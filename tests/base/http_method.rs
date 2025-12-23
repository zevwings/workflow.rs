//! Base HTTP Method 模块测试
//!
//! 测试 HTTP 方法枚举的核心功能，包括 FromStr 和 Display trait。

use pretty_assertions::assert_eq;
use std::str::FromStr;
use workflow::base::http::HttpMethod;

#[test]
fn test_http_method_from_str_get() {
    let method = HttpMethod::from_str("GET").unwrap();
    match method {
        HttpMethod::Get => assert!(true),
        _ => panic!("Expected Get"),
    }
}

#[test]
fn test_http_method_from_str_post() {
    let method = HttpMethod::from_str("POST").unwrap();
    match method {
        HttpMethod::Post => assert!(true),
        _ => panic!("Expected Post"),
    }
}

#[test]
fn test_http_method_from_str_put() {
    let method = HttpMethod::from_str("PUT").unwrap();
    match method {
        HttpMethod::Put => assert!(true),
        _ => panic!("Expected Put"),
    }
}

#[test]
fn test_http_method_from_str_delete() {
    let method = HttpMethod::from_str("DELETE").unwrap();
    match method {
        HttpMethod::Delete => assert!(true),
        _ => panic!("Expected Delete"),
    }
}

#[test]
fn test_http_method_from_str_patch() {
    let method = HttpMethod::from_str("PATCH").unwrap();
    match method {
        HttpMethod::Patch => assert!(true),
        _ => panic!("Expected Patch"),
    }
}

#[test]
fn test_http_method_from_str_invalid() {
    let result = HttpMethod::from_str("INVALID");
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid HTTP method"));
}

#[test]
fn test_http_method_from_str_case_sensitive() {
    // 测试大小写敏感
    assert!(HttpMethod::from_str("get").is_err());
    assert!(HttpMethod::from_str("post").is_err());
    assert!(HttpMethod::from_str("Get").is_err());
}

#[test]
fn test_http_method_display_get() {
    assert_eq!(format!("{}", HttpMethod::Get), "GET");
}

#[test]
fn test_http_method_display_post() {
    assert_eq!(format!("{}", HttpMethod::Post), "POST");
}

#[test]
fn test_http_method_display_put() {
    assert_eq!(format!("{}", HttpMethod::Put), "PUT");
}

#[test]
fn test_http_method_display_delete() {
    assert_eq!(format!("{}", HttpMethod::Delete), "DELETE");
}

#[test]
fn test_http_method_display_patch() {
    assert_eq!(format!("{}", HttpMethod::Patch), "PATCH");
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
