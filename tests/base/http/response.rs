//! HTTP Response 测试
//!
//! 测试 HTTP 响应的解析和处理功能。
//!
//! ## 测试策略
//!
//! - 所有测试返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 使用 `MockServer` 模拟 HTTP 服务器
//! - 测试各种响应场景：成功、错误、不同内容类型

use crate::common::http_helpers::MockServer;
use color_eyre::Result;
use serde::Deserialize;
use serde_json::Value;
use workflow::base::http::{HttpClient, RequestConfig};

/// 设置 Mock 服务器
fn setup_mock_server() -> MockServer {
    MockServer::new()
}

#[derive(Debug, Deserialize, PartialEq)]
struct TestData {
    id: u32,
    name: String,
}

// ==================== Response Status Tests ====================

#[test]
fn test_response_is_success_with_200_status_returns_true() -> Result<()> {
    // Arrange: 准备 Mock 服务器和成功响应
    let mut mock_server = setup_mock_server();
    let url = format!("{}/success", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/success")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "ok"}"#)
        .create();

    // Act: 发送请求并获取响应
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    // Assert: 验证响应状态为成功
    assert!(response.is_success());
    assert!(!response.is_error());
    assert_eq!(response.status, 200);
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_is_error_with_404_status_returns_true() -> Result<()> {
    // Arrange: 准备 Mock 服务器和错误响应
    let mut mock_server = setup_mock_server();
    let url = format!("{}/error", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/error")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Not Found"}"#)
        .create();

    // Act: 发送请求并获取响应
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    // Assert: 验证响应状态为错误
    assert!(!response.is_success());
    assert!(response.is_error());
    assert_eq!(response.status, 404);
    _mock.assert();
    Ok(())
}

// ==================== Response Content Parsing Tests ====================

#[test]
fn test_response_as_json_with_valid_json_returns_parsed_data() -> Result<()> {
    // Arrange: 准备 Mock 服务器和 JSON 响应
    let mut mock_server = setup_mock_server();
    let url = format!("{}/json", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id": 1, "name": "test"}"#)
        .create();

    // Act: 发送请求并解析 JSON
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;
    let data: TestData = response.as_json()?;

    // Assert: 验证 JSON 解析结果正确
    assert_eq!(data.id, 1);
    assert_eq!(data.name, "test");
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_as_json_value_with_valid_json_returns_value() -> Result<()> {
    // Arrange: 准备 Mock 服务器和 JSON 响应
    let mut mock_server = setup_mock_server();
    let url = format!("{}/json-value", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/json-value")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"key": "value", "number": 42}"#)
        .create();

    // Act: 发送请求并解析 JSON Value
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;
    let json: Value = response.as_json()?;

    // Assert: 验证 JSON Value 内容正确
    assert_eq!(json["key"], "value");
    assert_eq!(json["number"], 42);
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_as_text_with_text_response_returns_text() -> Result<()> {
    // Arrange: 准备 Mock 服务器和文本响应
    let mut mock_server = setup_mock_server();
    let url = format!("{}/text", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/text")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("Hello, World!")
        .create();

    // Act: 发送请求并获取文本
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;
    let text = response.as_text()?;

    // Assert: 验证文本内容正确
    assert_eq!(text, "Hello, World!");
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_as_bytes_with_binary_response_returns_bytes() -> Result<()> {
    // Arrange: 准备 Mock 服务器和二进制响应
    let mut mock_server = setup_mock_server();
    let url = format!("{}/bytes", mock_server.base_url);
    let body_bytes = b"binary data\x00\x01\x02";

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/bytes")
        .with_status(200)
        .with_header("content-type", "application/octet-stream")
        .with_body(body_bytes)
        .create();

    // Act: 发送请求并获取字节
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;
    let bytes = response.as_bytes();

    // Assert: 验证字节内容正确
    assert_eq!(bytes, body_bytes);
    _mock.assert();
    Ok(())
}

// ==================== Response Success Validation Tests ====================

#[test]
fn test_response_ensure_success_with_success_response_returns_ok() -> Result<()> {
    // Arrange: 准备 Mock 服务器和成功响应
    let mut mock_server = setup_mock_server();
    let url = format!("{}/success", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/success")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "ok"}"#)
        .create();

    // Act: 发送请求并验证成功
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;
    let result = response.ensure_success();

    // Assert: 验证返回 Ok
    assert!(result.is_ok());
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_ensure_success_with_error_response_returns_error() -> Result<()> {
    // Arrange: 准备 Mock 服务器和错误响应
    let mut mock_server = setup_mock_server();
    let url = format!("{}/error", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/error")
        .with_status(500)
        .with_header("content-type", "text/plain")
        .with_body("Internal Server Error")
        .create();

    // Act: 发送请求并验证成功（应该失败）
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;
    let result = response.ensure_success();

    // Assert: 验证返回错误且错误消息包含状态码
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("500"));
        assert!(
            error_msg.contains("500")
                || error_msg.contains("Internal Server Error")
                || error_msg.contains("Unable to read response body")
        );
    }
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_ensure_success_with_custom_error_handler_returns_ok() -> Result<()> {
    // Arrange: 准备 Mock 服务器和成功响应
    let mut mock_server = setup_mock_server();
    let url = format!("{}/success", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/success")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "ok"}"#)
        .create();

    // Act: 发送请求并使用自定义错误处理器验证成功
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;
    let result = response
        .ensure_success_with(|r| color_eyre::eyre::eyre!("Custom error: status {}", r.status));

    // Assert: 验证返回 Ok
    assert!(result.is_ok());
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_ensure_success_with_custom_error_handler_on_error_returns_error() -> Result<()> {
    // Arrange: 准备 Mock 服务器和错误响应
    let mut mock_server = setup_mock_server();
    let url = format!("{}/error", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/error")
        .with_status(403)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Forbidden"}"#)
        .create();

    // Act: 发送请求并使用自定义错误处理器验证成功（应该失败）
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;
    let result = response
        .ensure_success_with(|r| color_eyre::eyre::eyre!("Custom error: status {}", r.status));

    // Assert: 验证返回错误且错误消息包含状态码
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("403"));
    }
    _mock.assert();
    Ok(())
}

// ==================== Response Error Message Extraction Tests ====================

#[test]
fn test_response_extract_error_message_with_json_error_returns_message() -> Result<()> {
    // Arrange: 准备 Mock 服务器和 JSON 错误响应
    let mut mock_server = setup_mock_server();
    let url = format!("{}/error-json", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/error-json")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": {"message": "Invalid input"}, "code": 400}"#)
        .create();

    // Act: 发送请求并提取错误消息
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;
    let error_msg = response.extract_error_message();

    // Assert: 验证错误消息包含预期内容
    assert!(error_msg.contains("Invalid input"));
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_extract_error_message_with_text_error_returns_message() -> Result<()> {
    // Arrange: 准备 Mock 服务器和文本错误响应
    let mut mock_server = setup_mock_server();
    let url = format!("{}/error-text", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/error-text")
        .with_status(500)
        .with_header("content-type", "text/plain")
        .with_body("Server Error: Database connection failed")
        .create();

    // Act: 发送请求并提取错误消息
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;
    let error_msg = response.extract_error_message();

    // Assert: 验证错误消息包含预期内容
    assert!(error_msg.contains("Database connection failed"));
    _mock.assert();
    Ok(())
}

// ==================== Response Metadata Tests ====================

#[test]
fn test_response_headers_with_custom_headers_returns_headers() -> Result<()> {
    // Arrange: 准备 Mock 服务器和自定义响应头
    let mut mock_server = setup_mock_server();
    let url = format!("{}/headers", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/headers")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_header("x-custom-header", "custom-value")
        .with_header("x-request-id", "12345")
        .with_body(r#"{}"#)
        .create();

    // Act: 发送请求并获取响应
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    // Assert: 验证响应头正确
    assert_eq!(
        response
            .headers
            .get("x-custom-header")
            .ok_or_else(|| color_eyre::eyre::eyre!("Missing header"))?,
        "custom-value"
    );
    assert_eq!(
        response
            .headers
            .get("x-request-id")
            .ok_or_else(|| color_eyre::eyre::eyre!("Missing header"))?,
        "12345"
    );
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_status_text_with_201_status_returns_status_text() -> Result<()> {
    // Arrange: 准备 Mock 服务器和 201 状态码
    let mut mock_server = setup_mock_server();
    let url = format!("{}/status", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/status")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{}"#)
        .create();

    // Act: 发送请求并获取响应
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    // Assert: 验证状态码和状态文本
    assert_eq!(response.status, 201);
    assert!(!response.status_text.is_empty());
    _mock.assert();
    Ok(())
}

// ==================== Response Multiple Parsing Tests ====================

#[test]
fn test_response_multiple_parses_with_same_response_allows_multiple_parses() -> Result<()> {
    // Arrange: 准备 Mock 服务器和 JSON 响应
    let mut mock_server = setup_mock_server();
    let url = format!("{}/multi-parse", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/multi-parse")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id": 42, "name": "test"}"#)
        .create();

    // Act: 发送请求并多次解析响应体
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;
    let json1: Value = response.as_json()?;
    let json2: Value = response.as_json()?;
    let text = response.as_text()?;

    // Assert: 验证多次解析结果一致
    assert_eq!(json1["id"], 42);
    assert_eq!(json2["id"], 42);
    assert!(text.contains("42"));
    _mock.assert();
    Ok(())
}

// ==================== Response Error Handling Tests ====================

#[test]
fn test_response_ensure_success_with_invalid_utf8_body_returns_error() -> Result<()> {
    // Arrange: 准备 Mock 服务器和无效 UTF-8 响应体
    let mut mock_server = setup_mock_server();
    let url = format!("{}/invalid-utf8", mock_server.base_url);
    let invalid_utf8_body = vec![0xFF, 0xFE, 0xFD];

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/invalid-utf8")
        .with_status(500)
        .with_header("content-type", "application/octet-stream")
        .with_body(invalid_utf8_body)
        .create();

    // Act: 发送请求并验证成功（应该失败）
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;
    let result = response.ensure_success();

    // Assert: 验证返回错误且错误消息包含状态码和回退消息
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("500"));
        assert!(error_msg.contains("Unable to read response body"));
    }
    _mock.assert();
    Ok(())
}

// ==================== Additional Response Tests ====================

#[test]
fn test_http_response_is_success_with_201_status_returns_true() -> color_eyre::Result<()> {
    // Arrange: 准备 Mock 服务器和 201 响应
    let mut manager = MockServer::new();

    let mock = manager
        .server()
        .mock("POST", "/test")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"created": true}"#)
        .create();

    // Act: 发送 POST 请求并获取响应
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.post(&url, config)?;

    // Assert: 验证 201 状态码被视为成功
    assert_eq!(response.status, 201);
    assert!(response.is_success());

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_is_success_with_299_status_returns_true() -> color_eyre::Result<()> {
    // Arrange: 准备 Mock 服务器和 299 响应
    let mut manager = MockServer::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(299)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "success"}"#)
        .create();

    // Act: 发送 GET 请求并获取响应
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    // Assert: 验证 299 状态码被视为成功
    assert_eq!(response.status, 299);
    assert!(response.is_success());

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_debug_with_valid_response_returns_debug_string() -> color_eyre::Result<()> {
    // Arrange: 准备 Mock 服务器和响应
    let mut manager = MockServer::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "OK"}"#)
        .create();

    // Act: 发送请求并格式化 Debug 输出
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;
    let debug_str = format!("{:?}", response);

    // Assert: 验证 Debug 字符串包含状态码或消息
    assert!(debug_str.contains("200") || debug_str.contains("OK"));

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_extract_error_message_with_message_field_returns_message() -> color_eyre::Result<()> {
    // Arrange: 准备 Mock 服务器和包含 message 字段的错误响应
    let mut manager = MockServer::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "Simple error message"}"#)
        .create();

    // Act: 发送请求并提取错误消息
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;
    let error_msg = response.extract_error_message();

    // Assert: 验证错误消息包含 message 字段或消息内容
    assert!(error_msg.contains("message") || error_msg.contains("Simple error message"));

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_extract_error_message_without_error_field_returns_full_json() -> color_eyre::Result<()> {
    // Arrange: 准备 Mock 服务器和没有 error/message 字段的错误响应
    let mut manager = MockServer::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"data": {"key": "value"}, "status": "error"}"#)
        .create();

    // Act: 发送请求并提取错误消息
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;
    let error_msg = response.extract_error_message();

    // Assert: 验证返回完整的 JSON 字符串（因为没有 error/message 字段）
    assert!(error_msg.contains("data") || error_msg.contains("status"));

    mock.assert();
    manager.cleanup();
    Ok(())
}
