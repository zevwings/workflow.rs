//! HTTP Config 测试
//!
//! 测试 HTTP 请求配置和 Multipart 请求配置的功能。
//!
//! ## 测试策略
//!
//! - 所有测试返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 测试配置构建器模式和方法链

use color_eyre::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use workflow::base::http::{Authorization, MultipartRequestConfig, RequestConfig};

// ==================== RequestConfig Creation Tests ====================

#[test]
fn test_request_config_new_with_no_parameters_creates_empty_config() {
    // Arrange: 准备创建新配置

    // Act: 创建新的 RequestConfig
    let config = RequestConfig::<Value, Value>::new();

    // Assert: 验证所有字段为 None
    assert!(config.body.is_none());
    assert!(config.query.is_none());
    assert!(config.auth.is_none());
    assert!(config.headers.is_none());
    assert!(config.timeout.is_none());
}

#[test]
fn test_request_config_default_with_no_parameters_creates_empty_config() {
    // Arrange: 准备创建默认配置

    // Act: 创建默认的 RequestConfig
    let config = RequestConfig::<Value, Value>::default();

    // Assert: 验证所有字段为 None
    assert!(config.body.is_none());
    assert!(config.query.is_none());
    assert!(config.auth.is_none());
    assert!(config.headers.is_none());
    assert!(config.timeout.is_none());
}

// ==================== RequestConfig Builder Tests ====================

#[test]
fn test_request_config_body_with_json_value_sets_body() -> Result<()> {
    // Arrange: 准备 JSON 请求体
    let body = serde_json::json!({"key": "value"});

    // Act: 设置请求体
    let config = RequestConfig::<Value, Value>::new().body(&body);

    // Assert: 验证请求体已设置
    assert!(config.body.is_some());
    if let Some(ref body) = config.body {
        assert_eq!(body["key"], "value");
    }
    Ok(())
}

#[test]
fn test_request_config_query_with_array_sets_query() {
    // Arrange: 准备查询参数数组
    let query = [("page", "1"), ("limit", "10")];

    // Act: 设置查询参数
    let config = RequestConfig::<Value, _>::new().query(&query);

    // Assert: 验证查询参数已设置
    assert!(config.query.is_some());
}

#[test]
fn test_request_config_query_with_hashmap_sets_query() {
    // Arrange: 准备查询参数 HashMap
    let mut params = HashMap::new();
    params.insert("state", "open");
    params.insert("sort", "created");

    // Act: 设置查询参数
    let config = RequestConfig::<Value, _>::new().query(&params);

    // Assert: 验证查询参数已设置
    assert!(config.query.is_some());
}

#[test]
fn test_request_config_auth_with_credentials_sets_auth() -> Result<()> {
    // Arrange: 准备认证信息
    let auth = Authorization::new("username", "password");

    // Act: 设置认证信息
    let config = RequestConfig::<Value, Value>::new().auth(&auth);

    // Assert: 验证认证信息已设置
    assert!(config.auth.is_some());
    if let Some(config_auth) = config.auth {
        assert_eq!(config_auth.username, "username");
        assert_eq!(config_auth.password, "password");
    }
    Ok(())
}

#[test]
fn test_request_config_headers_with_header_map_sets_headers() -> Result<()> {
    // Arrange: 准备请求头
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-Custom-Header", "value".parse()?);

    // Act: 设置请求头
    let config = RequestConfig::<Value, Value>::new().headers(&headers);

    // Assert: 验证请求头已设置
    assert!(config.headers.is_some());
    if let Some(headers) = config.headers {
        assert_eq!(
            headers
                .get("X-Custom-Header")
                .ok_or_else(|| color_eyre::eyre::eyre!("Missing header"))?,
            "value"
        );
    }
    Ok(())
}

#[test]
fn test_request_config_timeout_with_duration_sets_timeout() -> Result<()> {
    // Arrange: 准备超时时间
    let timeout = Duration::from_secs(60);

    // Act: 设置超时时间
    let config = RequestConfig::<Value, Value>::new().timeout(timeout);

    // Assert: 验证超时时间已设置
    assert!(config.timeout.is_some());
    if let Some(t) = config.timeout {
        assert_eq!(t, timeout);
    }
    Ok(())
}

#[test]
fn test_request_config_chain_with_multiple_options_sets_all_options() -> Result<()> {
    // Arrange: 准备所有配置选项
    let body = serde_json::json!({"data": "test"});
    let query = [("filter", "active")];
    let auth = Authorization::new("user", "pass");
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-Request-ID", "12345".parse()?);
    let timeout = Duration::from_secs(30);

    // Act: 链式设置所有配置选项
    let config = RequestConfig::new()
        .body(&body)
        .query(&query)
        .auth(&auth)
        .headers(&headers)
        .timeout(timeout);

    // Assert: 验证所有配置选项已设置
    assert!(config.body.is_some());
    assert!(config.query.is_some());
    assert!(config.auth.is_some());
    assert!(config.headers.is_some());
    assert!(config.timeout.is_some());
    Ok(())
}

// ==================== MultipartRequestConfig Creation Tests ====================

#[test]
fn test_multipart_request_config_new_with_no_parameters_creates_empty_config() {
    // Arrange: 准备创建新配置

    // Act: 创建新的 MultipartRequestConfig
    let config = MultipartRequestConfig::<Value>::new();

    // Assert: 验证所有字段为 None
    assert!(config.multipart.is_none());
    assert!(config.query.is_none());
    assert!(config.auth.is_none());
    assert!(config.headers.is_none());
    assert!(config.timeout.is_none());
}

#[test]
fn test_multipart_request_config_default_with_no_parameters_creates_empty_config() {
    // Arrange: 准备创建默认配置

    // Act: 创建默认的 MultipartRequestConfig
    let config = MultipartRequestConfig::<Value>::default();

    // Assert: 验证所有字段为 None
    assert!(config.multipart.is_none());
    assert!(config.query.is_none());
    assert!(config.auth.is_none());
    assert!(config.headers.is_none());
    assert!(config.timeout.is_none());
}

// ==================== MultipartRequestConfig Builder Tests ====================

#[test]
fn test_multipart_request_config_multipart_with_form_sets_multipart() {
    // Arrange: 准备 multipart 表单
    let form = reqwest::blocking::multipart::Form::new()
        .text("key", "value")
        .text("name", "test");

    // Act: 设置 multipart 表单
    let config = MultipartRequestConfig::<Value>::new().multipart(form);

    // Assert: 验证 multipart 表单已设置
    assert!(config.multipart.is_some());
}

#[test]
fn test_multipart_request_config_query_with_json_sets_query() {
    // Arrange: 准备查询参数 JSON
    let query = serde_json::json!({"param": "value"});

    // Act: 设置查询参数
    let config = MultipartRequestConfig::new().query(query);

    // Assert: 验证查询参数已设置
    assert!(config.query.is_some());
}

#[test]
fn test_multipart_request_config_auth_with_credentials_sets_auth() -> Result<()> {
    // Arrange: 准备认证信息
    let auth = Authorization::new("username", "password");
    let auth_clone = auth.clone();

    // Act: 设置认证信息
    let config = MultipartRequestConfig::<Value>::new().auth(auth);

    // Assert: 验证认证信息已设置
    assert!(config.auth.is_some());
    if let Some(config_auth) = config.auth {
        assert_eq!(config_auth.username, auth_clone.username);
        assert_eq!(config_auth.password, auth_clone.password);
    }
    Ok(())
}

#[test]
fn test_multipart_request_config_headers_with_header_map_sets_headers() -> Result<()> {
    // Arrange: 准备请求头
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-Custom-Header", "value".parse()?);

    // Act: 设置请求头
    let config = MultipartRequestConfig::<Value>::new().headers(headers.clone());

    // Assert: 验证请求头已设置
    assert!(config.headers.is_some());
    if let Some(headers) = config.headers {
        assert_eq!(
            headers
                .get("X-Custom-Header")
                .ok_or_else(|| color_eyre::eyre::eyre!("Missing header"))?,
            "value"
        );
    }
    Ok(())
}

#[test]
fn test_multipart_request_config_timeout_with_duration_sets_timeout() -> Result<()> {
    // Arrange: 准备超时时间
    let timeout = Duration::from_secs(120);

    // Act: 设置超时时间
    let config = MultipartRequestConfig::<Value>::new().timeout(timeout);

    // Assert: 验证超时时间已设置
    assert!(config.timeout.is_some());
    if let Some(t) = config.timeout {
        assert_eq!(t, timeout);
    }
    Ok(())
}

#[test]
fn test_multipart_request_config_chain_with_multiple_options_sets_all_options() -> Result<()> {
    // Arrange: 准备所有配置选项
    let form = reqwest::blocking::multipart::Form::new().text("file", "content");
    let query = serde_json::json!({"param": "value"});
    let auth = Authorization::new("user", "pass");
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-Upload-ID", "67890".parse()?);
    let timeout = Duration::from_secs(60);

    // Act: 链式设置所有配置选项
    let config = MultipartRequestConfig::new()
        .multipart(form)
        .query(query)
        .auth(auth)
        .headers(headers)
        .timeout(timeout);

    // Assert: 验证所有配置选项已设置
    assert!(config.multipart.is_some());
    assert!(config.query.is_some());
    assert!(config.auth.is_some());
    assert!(config.headers.is_some());
    assert!(config.timeout.is_some());
    Ok(())
}
