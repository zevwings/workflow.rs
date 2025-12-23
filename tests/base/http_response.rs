//! Base HTTP Response 模块测试
//!
//! 测试 HTTP 响应的核心功能，包括 HttpResponse 结构体。
//! 使用 mock server 来创建真实的 HTTP 响应进行测试。

use pretty_assertions::assert_eq;
use serde_json::Value;
use workflow::base::http::{HttpClient, RequestConfig};
use crate::common::mock_server::MockServerManager;

#[test]
fn test_http_response_is_success_200() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "success"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    assert_eq!(response.status, 200);
    assert!(response.is_success());
    assert!(!response.is_error());

    mock.assert();
    manager.cleanup();
    Ok(())
}

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
fn test_http_response_is_error_404() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Not Found"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    assert_eq!(response.status, 404);
    assert!(!response.is_success());
    assert!(response.is_error());

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_is_error_500() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    assert_eq!(response.status, 500);
    assert!(!response.is_success());
    assert!(response.is_error());

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_as_bytes() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

    let body = b"test response body";
    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body(body)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    assert_eq!(response.as_bytes(), body);

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_as_text() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

    let body = "Hello, World!";
    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body(body)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    let text = response.as_text()?;
    assert_eq!(text, body);

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_as_json() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"key": "value", "number": 42}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    let json: Value = response.as_json()?;
    assert_eq!(json["key"], "value");
    assert_eq!(json["number"], 42);

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_ensure_success_200() -> color_eyre::Result<()> {
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

    let result = response.ensure_success()?;
    assert_eq!(result.status, 200);

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_ensure_success_404() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Not Found"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    let result = response.ensure_success();
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("404"));

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_ensure_success_with_custom_handler() -> color_eyre::Result<()> {
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

    let result = response.ensure_success_with(|r| {
        color_eyre::eyre::eyre!("Custom error: {}", r.status)
    })?;
    assert_eq!(result.status, 200);

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_ensure_success_with_custom_handler_error() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Server Error"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    let result = response.ensure_success_with(|r| {
        color_eyre::eyre::eyre!("Custom error: {}", r.status)
    });
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("500"));

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_extract_error_message_json() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": {"message": "Custom error"}}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    let error_msg = response.extract_error_message();
    assert!(error_msg.contains("error") || error_msg.contains("Custom error"));

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_response_extract_error_message_text() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

    let body = "Plain text error message";
    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(400)
        .with_header("content-type", "text/plain")
        .with_body(body)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    let error_msg = response.extract_error_message();
    assert_eq!(error_msg, body);

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
