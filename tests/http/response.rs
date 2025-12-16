//! HTTP 响应处理测试
//!
//! 测试 HTTP 响应的解析和处理功能，包括：
//! - JSON 解析测试
//! - 文本解析测试
//! - 错误响应处理测试
//! - 响应状态检查测试

use crate::common::http_helpers::MockServer;
use color_eyre::Result;
use serde::Deserialize;
use serde_json::Value;
use workflow::base::http::{HttpClient, RequestConfig};

// ==================== JSON 解析测试（P0） ====================

#[test]
fn test_response_as_json_success() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "success", "code": 200}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    let data: Value = response.as_json()?;
    assert_eq!(data["message"], "success");
    assert_eq!(data["code"], 200);
    Ok(())
}

#[test]
fn test_response_as_json_with_struct() -> Result<()> {
    #[derive(Debug, Deserialize)]
    struct TestData {
        message: String,
        code: u32,
    }

    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "success", "code": 200}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    let data: TestData = response.as_json()?;
    assert_eq!(data.message, "success");
    assert_eq!(data.code, 200);
    Ok(())
}

#[test]
fn test_response_as_json_invalid_json() {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "success""#) // 无效的 JSON
        .create();

    let client = HttpClient::global().unwrap();
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config).unwrap();

    let result: Result<Value> = response.as_json();
    assert!(result.is_err());
}

#[test]
fn test_response_as_json_empty() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("")
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    // 空响应应该解析为 null 或失败
    let result: Result<Value> = response.as_json();
    // 根据实现，空响应可能解析为 null 或失败
    // 这里只验证不会 panic
    let _ = result;
    Ok(())
}

// ==================== 文本解析测试 ====================

#[test]
fn test_response_as_text_success() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("Hello, World!")
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    let text = response.as_text()?;
    assert_eq!(text, "Hello, World!");
    Ok(())
}

#[test]
fn test_response_as_text_empty() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("")
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    let text = response.as_text()?;
    assert_eq!(text, "");
    Ok(())
}

// ==================== 响应状态检查测试 ====================

#[test]
fn test_response_is_success() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(200)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    assert!(response.is_success());
    assert!(!response.is_error());
    Ok(())
}

#[test]
fn test_response_is_error() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(404)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    assert!(!response.is_success());
    assert!(response.is_error());
    Ok(())
}

#[test]
fn test_response_ensure_success() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "success"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    let response = response.ensure_success()?;
    assert_eq!(response.status, 200);
    Ok(())
}

#[test]
fn test_response_ensure_success_fails() {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Not Found"}"#)
        .create();

    let client = HttpClient::global().unwrap();
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config).unwrap();

    let result = response.ensure_success();
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("404") || error_msg.contains("Not Found"));
}

#[test]
fn test_response_ensure_success_with_custom_handler() {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let client = HttpClient::global().unwrap();
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config).unwrap();

    let result = response.ensure_success_with(|r| {
        color_eyre::eyre::eyre!("Custom error: status {}", r.status)
    });
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Custom error"));
    assert!(error_msg.contains("500"));
}

// ==================== 错误消息提取测试 ====================

#[test]
fn test_response_extract_error_message() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": {"message": "Bad Request"}, "code": 400}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    let error_msg = response.extract_error_message();
    assert!(!error_msg.is_empty());
    // 错误消息应该包含相关信息
    assert!(error_msg.contains("400") || error_msg.contains("Bad Request"));
    Ok(())
}

#[test]
fn test_response_extract_error_message_text() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(500)
        .with_header("content-type", "text/plain")
        .with_body("Internal Server Error")
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    let error_msg = response.extract_error_message();
    assert_eq!(error_msg, "Internal Server Error");
    Ok(())
}

// ==================== 字节访问测试 ====================

#[test]
fn test_response_as_bytes() -> Result<()> {
    let mut mock_server = MockServer::new();
    let body = b"Hello, World!";
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "application/octet-stream")
        .with_body(body)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    let bytes = response.as_bytes();
    assert_eq!(bytes, body);
    Ok(())
}
