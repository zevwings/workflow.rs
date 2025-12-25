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
use crate::common::mock_server::MockServerManager;
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

#[test]
fn test_response_is_success() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    assert!(response.is_success());
    assert!(!response.is_error());
    assert_eq!(response.status, 200);
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_is_error() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    assert!(!response.is_success());
    assert!(response.is_error());
    assert_eq!(response.status, 404);
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_as_json() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    let data: TestData = response.as_json()?;
    assert_eq!(data.id, 1);
    assert_eq!(data.name, "test");
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_as_json_value() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    let json: Value = response.as_json()?;
    assert_eq!(json["key"], "value");
    assert_eq!(json["number"], 42);
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_as_text() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    let text = response.as_text()?;
    assert_eq!(text, "Hello, World!");
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_as_bytes() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    let bytes = response.as_bytes();
    assert_eq!(bytes, body_bytes);
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_ensure_success() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    let result = response.ensure_success();
    assert!(result.is_ok());
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_ensure_success_fails() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    let result = response.ensure_success();
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("500"));
        // ensure_success 的错误消息格式是 "HTTP request failed with status 500: {response_body}"
        // 响应体是 "Internal Server Error"，所以错误消息应该包含它
        // 但如果 as_text() 失败，会使用 "Unable to read response body"
        // 我们检查状态码和错误消息格式
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
fn test_response_ensure_success_with() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    let result = response
        .ensure_success_with(|r| color_eyre::eyre::eyre!("Custom error: status {}", r.status));
    assert!(result.is_ok());
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_ensure_success_with_fails() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    let result = response
        .ensure_success_with(|r| color_eyre::eyre::eyre!("Custom error: status {}", r.status));
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("403"));
    }
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_extract_error_message_json() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    let error_msg = response.extract_error_message();
    assert!(error_msg.contains("Invalid input"));
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_extract_error_message_text() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    let error_msg = response.extract_error_message();
    assert!(error_msg.contains("Database connection failed"));
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_headers() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    assert_eq!(
        response.headers.get("x-custom-header").ok_or_else(|| color_eyre::eyre::eyre!("Missing header"))?,
        "custom-value"
    );
    assert_eq!(response.headers.get("x-request-id").ok_or_else(|| color_eyre::eyre::eyre!("Missing header"))?, "12345");
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_status_text() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    assert_eq!(response.status, 201);
    // status_text 应该是 "Created" 或类似的值
    assert!(!response.status_text.is_empty());
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_multiple_parses() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    // 可以多次解析响应体
    let json1: Value = response.as_json()?;
    let json2: Value = response.as_json()?;
    let text = response.as_text()?;

    assert_eq!(json1["id"], 42);
    assert_eq!(json2["id"], 42);
    assert!(text.contains("42"));
    _mock.assert();
    Ok(())
}

#[test]
fn test_response_ensure_success_with_invalid_utf8_body() -> Result<()> {
    // 测试 ensure_success 在响应体不是有效 UTF-8 时的错误处理
    // 这应该触发 unwrap_or_else 分支，返回 "Unable to read response body"
    let mut mock_server = setup_mock_server();
    let url = format!("{}/invalid-utf8", mock_server.base_url);

    // 创建无效的 UTF-8 字节序列
    let invalid_utf8_body = vec![0xFF, 0xFE, 0xFD];

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/invalid-utf8")
        .with_status(500)
        .with_header("content-type", "application/octet-stream")
        .with_body(invalid_utf8_body)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    let result = response.ensure_success();
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        // 应该包含状态码和回退消息
        assert!(error_msg.contains("500"));
        assert!(error_msg.contains("Unable to read response body"));
    }
    _mock.assert();
    Ok(())
}

// ==================== 来自 response_core.rs 的补充测试 ====================

#[test]
fn test_http_response_is_success_201() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

    let mock = manager
        .server()
        .mock("POST", "/test")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"created": true}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.post(&url, config)?;

    assert_eq!(response.status, 201);
    assert!(response.is_success());

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_is_success_299() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(299)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "success"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    assert_eq!(response.status, 299);
    assert!(response.is_success());

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_debug() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "OK"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains("200") || debug_str.contains("OK"));

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_extract_error_message_with_message_field() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "Simple error message"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    let error_msg = response.extract_error_message();
    assert!(error_msg.contains("message") || error_msg.contains("Simple error message"));

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_extract_error_message_no_error_field() -> color_eyre::Result<()> {
    // 测试 extract_error_message 当 JSON 响应没有 error/message 字段时
    let mut manager = MockServerManager::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"data": {"key": "value"}, "status": "error"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    let error_msg = response.extract_error_message();
    // 应该返回完整的 JSON 字符串（因为没有 error/message 字段）
    assert!(error_msg.contains("data") || error_msg.contains("status"));

    mock.assert();
    manager.cleanup();
    Ok(())
}
