//! HTTP Response 测试
//!
//! 测试 HTTP 响应的解析和处理功能。
//!
//! ## 测试策略
//!
//! - 所有测试返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 使用 `MockServer` 模拟 HTTP 服务器
//! - 测试各种响应场景：成功、错误、不同内容类型

use crate::common::http_helpers::{setup_mock_server, MockServer};
use color_eyre::Result;
use serde::Deserialize;
use serde_json::Value;
use workflow::base::http::{HttpClient, RequestConfig};

#[derive(Debug, Deserialize, PartialEq)]
struct TestData {
    id: u32,
    name: String,
}

// ==================== Response Status Tests ====================

/// 测试响应成功状态判断（200状态码）
///
/// ## 测试目的
/// 验证 HTTP 响应能够正确判断成功状态（200状态码）。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 200 状态码
/// 2. 发送请求并获取响应
/// 3. 验证响应状态判断方法
///
/// ## 预期结果
/// - is_success() 返回 true
/// - is_error() 返回 false
/// - status 为 200
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

/// 测试响应错误状态判断（404状态码）
///
/// ## 测试目的
/// 验证 HTTP 响应能够正确判断错误状态（404状态码）。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 404 状态码
/// 2. 发送请求并获取响应
/// 3. 验证响应状态判断方法
///
/// ## 预期结果
/// - is_success() 返回 false
/// - is_error() 返回 true
/// - status 为 404
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

/// 测试解析有效的 JSON 响应为结构体
///
/// ## 测试目的
/// 验证 HTTP 响应能够正确解析 JSON 内容为指定的结构体类型。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 JSON 响应
/// 2. 发送请求并获取响应
/// 3. 使用 as_json() 解析为 TestData 结构体
///
/// ## 预期结果
/// - JSON 解析成功
/// - 结构体字段值正确
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

/// 测试解析有效的 JSON 响应为 Value
///
/// ## 测试目的
/// 验证 HTTP 响应能够正确解析 JSON 内容为 serde_json::Value。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 JSON 响应
/// 2. 发送请求并获取响应
/// 3. 使用 as_json() 解析为 Value
///
/// ## 预期结果
/// - JSON 解析成功
/// - Value 内容正确
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

/// 测试解析文本响应
///
/// ## 测试目的
/// 验证 HTTP 响应能够正确解析文本内容。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回文本响应
/// 2. 发送请求并获取响应
/// 3. 使用 as_text() 获取文本内容
///
/// ## 预期结果
/// - 文本内容正确
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

/// 测试解析二进制响应
///
/// ## 测试目的
/// 验证 HTTP 响应能够正确解析二进制内容。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回二进制响应
/// 2. 发送请求并获取响应
/// 3. 使用 as_bytes() 获取字节内容
///
/// ## 预期结果
/// - 字节内容正确
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

/// 测试确保响应成功的验证（成功响应）
///
/// ## 测试目的
/// 验证 ensure_success() 方法在成功响应时返回 Ok。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 200 状态码
/// 2. 发送请求并获取响应
/// 3. 调用 ensure_success() 验证
///
/// ## 预期结果
/// - ensure_success() 返回 Ok
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

/// 测试确保响应成功的验证（错误响应）
///
/// ## 测试目的
/// 验证 ensure_success() 方法在错误响应时返回错误。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 500 状态码
/// 2. 发送请求并获取响应
/// 3. 调用 ensure_success() 验证
///
/// ## 预期结果
/// - ensure_success() 返回错误
/// - 错误消息包含状态码
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

/// 测试使用自定义错误处理器的成功验证（成功响应）
///
/// ## 测试目的
/// 验证 ensure_success_with() 方法在成功响应时返回 Ok。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 200 状态码
/// 2. 发送请求并获取响应
/// 3. 使用自定义错误处理器调用 ensure_success_with() 验证
///
/// ## 预期结果
/// - ensure_success_with() 返回 Ok
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

/// 测试使用自定义错误处理器的成功验证（错误响应）
///
/// ## 测试目的
/// 验证 ensure_success_with() 方法在错误响应时调用自定义错误处理器并返回错误。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 403 状态码
/// 2. 发送请求并获取响应
/// 3. 使用自定义错误处理器调用 ensure_success_with() 验证
///
/// ## 预期结果
/// - ensure_success_with() 返回错误
/// - 错误消息包含状态码（由自定义处理器生成）
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

/// 测试从 JSON 错误响应中提取错误消息
///
/// ## 测试目的
/// 验证 extract_error_message() 方法能够从 JSON 错误响应中提取错误消息。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 JSON 错误响应
/// 2. 发送请求并获取响应
/// 3. 调用 extract_error_message() 提取错误消息
///
/// ## 预期结果
/// - 错误消息正确提取
/// - 消息包含预期的错误内容
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

/// 测试从文本错误响应中提取错误消息
///
/// ## 测试目的
/// 验证 extract_error_message() 方法能够从文本错误响应中提取错误消息。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回文本错误响应
/// 2. 发送请求并获取响应
/// 3. 调用 extract_error_message() 提取错误消息
///
/// ## 预期结果
/// - 错误消息正确提取
/// - 消息包含预期的错误内容
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

/// 测试获取响应头
///
/// ## 测试目的
/// 验证能够正确获取 HTTP 响应的自定义响应头。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回自定义响应头
/// 2. 发送请求并获取响应
/// 3. 获取响应头并验证
///
/// ## 预期结果
/// - 响应头正确获取
/// - 自定义响应头值正确
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

/// 测试获取状态码文本
///
/// ## 测试目的
/// 验证能够正确获取 HTTP 响应的状态码文本。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 201 状态码
/// 2. 发送请求并获取响应
/// 3. 验证状态码和状态文本
///
/// ## 预期结果
/// - 状态码为 201
/// - 状态文本不为空
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

/// 测试同一响应的多次解析
///
/// ## 测试目的
/// 验证能够对同一个响应进行多次解析（JSON、文本等）。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 JSON 响应
/// 2. 发送请求并获取响应
/// 3. 多次调用不同的解析方法
///
/// ## 预期结果
/// - 多次解析结果一致
/// - 不同解析方法都能正常工作
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

/// 测试无效 UTF-8 响应体的错误处理
///
/// ## 测试目的
/// 验证 ensure_success() 方法在响应体包含无效 UTF-8 时能够正确处理错误。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回包含无效 UTF-8 的响应体
/// 2. 发送请求并获取响应
/// 3. 调用 ensure_success() 验证
///
/// ## 预期结果
/// - ensure_success() 返回错误
/// - 错误处理正确（不会panic）
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

/// 测试响应成功状态判断（201状态码，补充测试）
///
/// ## 测试目的
/// 验证 HTTP 响应能够正确判断成功状态（201 Created状态码）。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 201 状态码
/// 2. 发送 POST 请求并获取响应
/// 3. 验证响应状态判断方法
///
/// ## 预期结果
/// - is_success() 返回 true
/// - status 为 201
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

/// 测试响应成功状态判断（299状态码，补充测试）
///
/// ## 测试目的
/// 验证 HTTP 响应能够正确判断成功状态（299状态码，2xx范围的上限）。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 299 状态码
/// 2. 发送 GET 请求并获取响应
/// 3. 验证响应状态判断方法
///
/// ## 预期结果
/// - is_success() 返回 true
/// - status 为 299
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

/// 测试响应调试格式化（补充测试）
///
/// ## 测试目的
/// 验证 HTTP 响应能够正确格式化调试字符串。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回响应
/// 2. 发送请求并获取响应
/// 3. 格式化 Debug 输出
///
/// ## 预期结果
/// - Debug 字符串包含状态码或消息
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

/// 测试从包含 message 字段的 JSON 错误响应中提取错误消息（补充测试）
///
/// ## 测试目的
/// 验证 extract_error_message() 方法能够从包含 message 字段的 JSON 错误响应中提取错误消息。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回包含 message 字段的 JSON 错误响应
/// 2. 发送请求并获取响应
/// 3. 调用 extract_error_message() 提取错误消息
///
/// ## 预期结果
/// - 错误消息正确提取
/// - 消息包含预期的错误内容
#[test]
fn test_http_response_extract_error_message_with_message_field() -> color_eyre::Result<()> {
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

/// 测试从没有 error 字段的 JSON 错误响应中提取错误消息（补充测试）
///
/// ## 测试目的
/// 验证 extract_error_message() 方法在没有 error 字段时能够返回完整的 JSON 字符串。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回没有 error 字段的 JSON 错误响应
/// 2. 发送请求并获取响应
/// 3. 调用 extract_error_message() 提取错误消息
///
/// ## 预期结果
/// - 错误消息包含完整的 JSON 内容
#[test]
fn test_http_response_extract_error_message_without_error_field() -> color_eyre::Result<()> {
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
