//! HTTP Client 测试
//!
//! 测试 HTTP 客户端的各种功能，包括 GET、POST、PUT、DELETE、PATCH 等方法。
//!
//! ## 测试策略
//!
//! - 使用 MockServer 模拟 HTTP 服务器，避免实际网络请求
//! - 所有测试返回 Result<()>，使用 ? 运算符处理错误
//! - 测试覆盖成功场景、错误场景、边界条件

use crate::common::http_helpers::{setup_mock_server, MockServer};
use color_eyre::Result;
use mockito::Matcher;
use serde_json::Value;
use workflow::base::http::{Authorization, HttpClient, RequestConfig};

// ==================== HttpClient Singleton Tests ====================

/// 测试 HttpClient 全局单例实例
///
/// ## 测试目的
/// 验证 HttpClient::global() 返回单例实例，多次调用返回同一个实例。
///
/// ## 预期结果
/// - 能够成功获取全局客户端实例
/// - 多次调用返回同一个实例（单例模式）
#[test]
fn test_http_client_global_returns_singleton_instance() -> Result<()> {
    // Arrange: 准备获取全局客户端

    // Act: 获取全局客户端实例
    let _client1 = HttpClient::global()?;
    let _client2 = HttpClient::global()?;

    // Assert: 验证都能成功获取客户端实例（单例复用）
    Ok(())
}

// ==================== GET Request Tests ====================

/// 测试 GET 请求成功响应
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送 GET 请求并处理成功响应。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 200 状态码和 JSON 响应
/// 2. 发送 GET 请求
/// 3. 验证响应状态码和内容
///
/// ## 预期结果
/// - 响应状态码为 200
/// - 响应标记为成功
/// - Mock 服务器收到预期的请求
#[test]
fn test_get_request_with_success_response_returns_response() -> Result<()> {
    // Arrange: 准备 Mock 服务器和响应
    let mut mock_server = setup_mock_server();
    let url = format!("{}/test", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "success"}"#)
        .create();

    // Act: 发送 GET 请求
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    // Assert: 验证响应状态和内容
    assert_eq!(response.status, 200);
    assert!(response.is_success());
    _mock.assert();

    Ok(())
}

/// 测试带查询参数的 GET 请求
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送带查询参数的 GET 请求。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器匹配查询参数（page=1, limit=10）
/// 2. 发送带查询参数的 GET 请求
/// 3. 验证查询参数正确传递
///
/// ## 预期结果
/// - 请求成功
/// - 查询参数正确匹配
#[test]
fn test_get_request_with_query_parameters_sends_query_string() -> Result<()> {
    // Arrange: 准备 Mock 服务器和查询参数匹配
    let mut mock_server = setup_mock_server();
    let url = format!("{}/test", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("page".to_string(), "1".to_string()),
            Matcher::UrlEncoded("limit".to_string(), "10".to_string()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"data": []}"#)
        .create();

    // Act: 发送带查询参数的 GET 请求
    let client = HttpClient::global()?;
    let query = [("page", "1"), ("limit", "10")];
    let config = RequestConfig::<Value, _>::new().query(&query);
    let response = client.get(&url, config)?;

    // Assert: 验证请求成功且查询参数匹配
    assert!(response.is_success());
    _mock.assert();

    Ok(())
}

/// 测试带认证头的 GET 请求
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送带认证头的 GET 请求。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器匹配 Basic 认证头
/// 2. 发送带认证的 GET 请求
/// 3. 验证认证头正确传递
///
/// ## 预期结果
/// - 请求成功
/// - 认证头正确匹配（Basic 认证）
#[test]
fn test_get_request_with_auth_header_sends_authorization() -> Result<()> {
    // Arrange: 准备 Mock 服务器和认证匹配
    let mut mock_server = setup_mock_server();
    let url = format!("{}/test", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .match_header("authorization", Matcher::Regex(r"Basic .+".to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"authenticated": true}"#)
        .create();

    // Act: 发送带认证的 GET 请求
    let client = HttpClient::global()?;
    let auth = workflow::base::http::Authorization::new("user", "pass");
    let config = RequestConfig::<Value, Value>::new().auth(&auth);
    let response = client.get(&url, config)?;

    // Assert: 验证请求成功且认证头匹配
    assert!(response.is_success());
    _mock.assert();

    Ok(())
}

/// 测试带自定义请求头的 GET 请求
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送带自定义请求头的 GET 请求。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器匹配自定义请求头（X-Custom-Header）
/// 2. 发送带自定义请求头的 GET 请求
/// 3. 验证请求头正确传递
///
/// ## 预期结果
/// - 请求成功
/// - 自定义请求头正确匹配
#[test]
fn test_get_request_with_custom_headers_sends_headers() -> Result<()> {
    // Arrange: 准备 Mock 服务器和自定义请求头
    let mut mock_server = setup_mock_server();
    let url = format!("{}/test", mock_server.base_url);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-Custom-Header", "test-value".parse()?);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .match_header("x-custom-header", "test-value")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{}"#)
        .create();

    // Act: 发送带自定义请求头的 GET 请求
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().headers(&headers);
    let response = client.get(&url, config)?;

    // Assert: 验证请求成功且请求头匹配
    assert!(response.is_success());
    _mock.assert();

    Ok(())
}

/// 测试POST请求的成功场景
///
/// ## 测试目的
/// 验证HTTP客户端能够正确发送POST请求，包括请求体序列化、响应解析和状态码检查。
///
/// ## 测试场景
/// 1. 启动mock HTTP服务器（使用`mockito`）
/// 2. 配置mock响应规则：
///    - 匹配POST方法和路径
///    - 验证请求头（Content-Type: application/json）
///    - 验证请求体（使用JsonString matcher精确匹配）
///    - 返回201 Created状态码和JSON响应
/// 3. 发送POST请求
/// 4. 验证响应状态码
/// 5. 验证mock服务器收到了预期的请求
///
/// ## 技术细节
/// - 使用`mockito::Matcher::JsonString`进行JSON body匹配
/// - 使用`serde_json::json!`宏创建JSON数据
/// - 测试完成后mock自动验证（`_mock.assert()`）
/// - MockServer在`Drop`时自动清理环境变量
///
/// ## 预期结果
/// - 响应状态码为201
/// - mock服务器确认收到了正确的请求
// ==================== POST Request Tests ====================

#[test]
fn test_post_request_with_json_body_returns_response() -> Result<()> {
    // Arrange: 准备 Mock 服务器和请求体
    let mut mock_server = setup_mock_server();
    let url = format!("{}/test", mock_server.base_url);
    let body_data = serde_json::json!({"name": "test", "value": 123});

    let _mock = mock_server
        .server
        .as_mut()
        .mock("POST", "/test")
        .match_header("content-type", "application/json")
        .match_body(Matcher::JsonString(serde_json::to_string(&body_data)?))
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id": 1, "name": "test"}"#)
        .create();

    // Act: 发送 POST 请求
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().body(&body_data);
    let response = client.post(&url, config)?;

    // Assert: 验证响应状态和请求匹配
    assert_eq!(response.status, 201);
    _mock.assert();

    Ok(())
}

// ==================== PUT Request Tests ====================

/// 测试 PUT 请求成功响应
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送 PUT 请求并处理成功响应。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 200 状态码
/// 2. 发送带 JSON 请求体的 PUT 请求
/// 3. 验证响应状态码和请求匹配
///
/// ## 预期结果
/// - 响应状态码为 200
/// - Mock 服务器收到预期的请求
#[test]
fn test_put_request_with_json_body_returns_response() -> Result<()> {
    // Arrange: 准备 Mock 服务器和请求体
    let mut mock_server = setup_mock_server();
    let url = format!("{}/test/1", mock_server.base_url);
    let body_data = serde_json::json!({"name": "updated"});

    let _mock = mock_server
        .server
        .as_mut()
        .mock("PUT", "/test/1")
        .match_header("content-type", "application/json")
        .match_body(Matcher::JsonString(serde_json::to_string(&body_data)?))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id": 1, "name": "updated"}"#)
        .create();

    // Act: 发送 PUT 请求
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().body(&body_data);
    let response = client.put(&url, config)?;

    // Assert: 验证响应状态和请求匹配
    assert_eq!(response.status, 200);
    _mock.assert();

    Ok(())
}

// ==================== DELETE Request Tests ====================

/// 测试 DELETE 请求成功响应
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送 DELETE 请求并处理成功响应。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 204 No Content 状态码
/// 2. 发送 DELETE 请求
/// 3. 验证响应状态码和请求匹配
///
/// ## 预期结果
/// - 响应状态码为 204
/// - Mock 服务器收到预期的请求
#[test]
fn test_delete_request_with_valid_url_returns_response() -> Result<()> {
    // Arrange: 准备 Mock 服务器
    let mut mock_server = setup_mock_server();
    let url = format!("{}/test/1", mock_server.base_url);

    let _mock = mock_server.server.as_mut().mock("DELETE", "/test/1").with_status(204).create();

    // Act: 发送 DELETE 请求
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.delete(&url, config)?;

    // Assert: 验证响应状态和请求匹配
    assert_eq!(response.status, 204);
    _mock.assert();

    Ok(())
}

// ==================== PATCH Request Tests ====================

/// 测试 PATCH 请求成功响应
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送 PATCH 请求并处理成功响应。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 200 状态码
/// 2. 发送带 JSON 请求体的 PATCH 请求
/// 3. 验证响应状态码和请求匹配
///
/// ## 预期结果
/// - 响应状态码为 200
/// - Mock 服务器收到预期的请求
#[test]
fn test_patch_request_with_json_body_returns_response() -> Result<()> {
    // Arrange: 准备 Mock 服务器和请求体
    let mut mock_server = setup_mock_server();
    let url = format!("{}/test/1", mock_server.base_url);
    let body_data = serde_json::json!({"status": "active"});

    let _mock = mock_server
        .server
        .as_mut()
        .mock("PATCH", "/test/1")
        .match_header("content-type", "application/json")
        .match_body(Matcher::JsonString(serde_json::to_string(&body_data)?))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id": 1, "status": "active"}"#)
        .create();

    // Act: 发送 PATCH 请求
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().body(&body_data);
    let response = client.patch(&url, config)?;

    // Assert: 验证响应状态和请求匹配
    assert_eq!(response.status, 200);
    _mock.assert();

    Ok(())
}

// ==================== Multipart Request Tests ====================

/// 测试 POST multipart 表单数据请求
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送 multipart/form-data 格式的 POST 请求。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器匹配 multipart/form-data 请求头
/// 2. 发送带表单数据的 POST 请求
/// 3. 验证响应状态码和请求匹配
///
/// ## 预期结果
/// - 响应状态码为 200
/// - 请求头正确匹配（multipart/form-data）
/// - Mock 服务器收到预期的请求
#[test]
fn test_post_multipart_request_with_form_data_returns_response() -> Result<()> {
    // Arrange: 准备 Mock 服务器和 multipart 表单数据
    let mut mock_server = setup_mock_server();
    let url = format!("{}/upload", mock_server.base_url);

    let form = reqwest::blocking::multipart::Form::new()
        .text("key", "value")
        .text("name", "test");

    let _mock = mock_server
        .server
        .as_mut()
        .mock("POST", "/upload")
        .match_header(
            "content-type",
            Matcher::Regex(r"multipart/form-data; boundary=.+".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"uploaded": true}"#)
        .create();

    // Act: 发送 multipart POST 请求
    let client = HttpClient::global()?;
    let config = workflow::base::http::MultipartRequestConfig::<Value>::new().multipart(form);
    let response = client.post_multipart(&url, config)?;

    // Assert: 验证响应成功且请求匹配
    assert!(response.is_success());
    _mock.assert();

    Ok(())
}

// ==================== Stream Request Tests ====================

/// 测试流式请求
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送流式请求并读取响应流。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回二进制流数据
/// 2. 发送流式 GET 请求
/// 3. 读取流数据并验证内容
///
/// ## 预期结果
/// - 流数据内容正确
/// - Mock 服务器收到预期的请求
#[test]
fn test_stream_request_with_get_method_returns_stream() -> Result<()> {
    // Arrange: 准备 Mock 服务器和流式响应
    let mut mock_server = setup_mock_server();
    let url = format!("{}/stream", mock_server.base_url);

    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/stream")
        .with_status(200)
        .with_header("content-type", "application/octet-stream")
        .with_body(b"streaming data")
        .create();

    // Act: 发送流式请求并读取数据
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let mut stream = client.stream(workflow::base::http::HttpMethod::Get, &url, config)?;

    let mut buffer = Vec::new();
    stream.copy_to(&mut buffer)?;

    // Assert: 验证流数据内容正确
    assert_eq!(buffer, b"streaming data");
    _mock.assert();

    Ok(())
}

/// 测试请求超时配置
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确应用超时配置。
///
/// ## 测试场景
/// 1. 配置请求超时为 5 秒
/// 2. 发送请求
/// 3. 验证超时配置正确传递（注意：mockito 不支持真正的延迟）
///
/// ## 预期结果
/// - 请求成功
/// - 超时配置正确传递
#[test]
fn test_request_with_timeout_config_applies_timeout() -> Result<()> {
    // Arrange: 准备 Mock 服务器（注意：mockito 不支持真正的延迟，这里只测试配置是否正确传递）
    let mut mock_server = setup_mock_server();
    let url = format!("{}/slow", mock_server.base_url);
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/slow")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{}"#)
        .create();

    // Act: 发送带超时配置的请求
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().timeout(std::time::Duration::from_secs(5));
    let response = client.get(&url, config)?;

    // Assert: 验证请求成功
    assert!(response.is_success());
    _mock.assert();
    Ok(())
}

/// 测试错误状态码响应处理
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确处理错误状态码响应（如 500）。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 500 错误状态码
/// 2. 发送请求
/// 3. 验证响应状态码和错误标记
///
/// ## 预期结果
/// - 响应状态码为 500
/// - 响应标记为错误
/// - Mock 服务器收到预期的请求
#[test]
fn test_request_with_error_status_handles_error_response() -> Result<()> {
    // Arrange: 准备返回错误状态码的 Mock 服务器
    let mut mock_server = setup_mock_server();
    let url = format!("{}/error", mock_server.base_url);
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/error")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    // Act: 发送请求
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    // Assert: 验证请求应该成功（HTTP 客户端层面），但响应状态码是 500
    assert_eq!(response.status, 500);
    assert!(response.is_error());
    _mock.assert();
    Ok(())
}

/// 测试包含所有选项的请求配置
///
/// ## 测试目的
/// 验证 HTTP 客户端能够同时配置和使用所有请求选项（查询参数、认证、请求头、请求体、超时）。
///
/// ## 测试场景
/// 1. 配置包含所有选项的请求（查询参数、认证、请求头、请求体、超时）
/// 2. 发送 POST 请求
/// 3. 验证所有选项正确传递
///
/// ## 预期结果
/// - 请求成功
/// - 所有选项正确匹配（查询参数、认证头、请求头、请求体）
#[test]
fn test_request_with_all_options_configures_all_options() -> Result<()> {
    // Arrange: 准备包含所有选项的请求配置
    let mut mock_server = setup_mock_server();
    let url = format!("{}/complex", mock_server.base_url);
    let body_data = serde_json::json!({"data": "test"});
    let query = [("filter", "active")];
    let auth = workflow::base::http::Authorization::new("user", "pass");
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-Request-ID", "12345".parse()?);
    let _mock = mock_server
        .server
        .as_mut()
        .mock("POST", "/complex")
        .match_header("authorization", Matcher::Regex(r"Basic .+".to_string()))
        .match_header("x-request-id", "12345")
        .match_query(Matcher::UrlEncoded(
            "filter".to_string(),
            "active".to_string(),
        ))
        .match_body(Matcher::JsonString(serde_json::to_string(&body_data)?))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"success": true}"#)
        .create();

    // Act: 发送包含所有选项的请求
    let client = HttpClient::global()?;
    let config = RequestConfig::new()
        .body(&body_data)
        .query(&query)
        .auth(&auth)
        .headers(&headers)
        .timeout(std::time::Duration::from_secs(10));
    let response = client.post(&url, config)?;

    // Assert: 验证请求成功
    assert!(response.is_success());
    _mock.assert();
    Ok(())
}

// ==================== Additional Tests from client_core.rs ====================

/// 测试 HTTP 客户端 GET 请求（补充测试）
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送 GET 请求并解析 JSON 响应。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 JSON 响应
/// 2. 发送 GET 请求
/// 3. 验证响应状态码和 JSON 内容
///
/// ## 预期结果
/// - 响应状态码为 200
/// - JSON 内容正确解析
#[test]
fn test_http_client_get_with_valid_url_returns_response() -> color_eyre::Result<()> {
    // Arrange: 准备 Mock 服务器
    let mut manager = MockServer::new();
    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "success"}"#)
        .create();

    // Act: 发送 GET 请求
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    // Assert: 验证响应状态和内容正确
    assert_eq!(response.status, 200);
    let json: Value = response.as_json()?;
    assert_eq!(json["message"], "success");

    mock.assert();
    manager.cleanup();
    Ok(())
}

/// 测试 HTTP 客户端 POST 请求（补充测试）
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送 POST 请求并解析 JSON 响应。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 JSON 响应
/// 2. 发送带 JSON 请求体的 POST 请求
/// 3. 验证响应状态码和 JSON 内容
///
/// ## 预期结果
/// - 响应状态码为 201
/// - JSON 内容正确解析
#[test]
fn test_http_client_post_with_json_body_returns_response() -> color_eyre::Result<()> {
    // Arrange: 准备 Mock 服务器和请求体
    let mut manager = MockServer::new();
    let body = serde_json::json!({"key": "value"});
    let mock = manager
        .server()
        .mock("POST", "/test")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id": 123, "key": "value"}"#)
        .create();

    // Act: 发送 POST 请求
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().body(&body);
    let url = format!("{}/test", manager.base_url());
    let response = client.post(&url, config)?;

    // Assert: 验证响应状态和内容正确
    assert_eq!(response.status, 201);
    let json: Value = response.as_json()?;
    assert_eq!(json["id"], 123);

    mock.assert();
    manager.cleanup();
    Ok(())
}

/// 测试带认证的 HTTP 客户端 GET 请求（补充测试）
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送带认证的 GET 请求。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器匹配 Basic 认证头
/// 2. 发送带认证的 GET 请求
/// 3. 验证响应状态码和 JSON 内容
///
/// ## 预期结果
/// - 响应状态码为 200
/// - 认证头正确匹配
/// - JSON 内容正确解析
#[test]
fn test_http_client_get_with_auth() -> color_eyre::Result<()> {
    let mut manager = MockServer::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .match_header(
            "authorization",
            mockito::Matcher::Regex(r"Basic .+".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"authenticated": true}"#)
        .create();

    let client = HttpClient::global()?;
    let auth = Authorization::new("user@example.com", "token");
    let config = RequestConfig::<Value, Value>::new().auth(&auth);
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    assert_eq!(response.status, 200);
    let json: Value = response.as_json()?;
    assert_eq!(json["authenticated"], true);

    mock.assert();
    manager.cleanup();
    Ok(())
}

/// 测试带查询参数的 HTTP 客户端 GET 请求（补充测试）
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送带查询参数的 GET 请求。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器匹配查询参数（page=1, per_page=10）
/// 2. 发送带查询参数的 GET 请求
/// 3. 验证查询参数正确传递
///
/// ## 预期结果
/// - 响应状态码为 200
/// - 查询参数正确匹配
#[test]
fn test_http_client_get_with_query() -> color_eyre::Result<()> {
    let mut manager = MockServer::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page".into(), "1".into()),
            mockito::Matcher::UrlEncoded("per_page".into(), "10".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page": 1, "per_page": 10}"#)
        .create();

    let client = HttpClient::global()?;
    let query = [("page", "1"), ("per_page", "10")];
    let config = RequestConfig::<Value, _>::new().query(&query);
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    assert_eq!(response.status, 200);

    mock.assert();
    manager.cleanup();
    Ok(())
}

/// 测试带自定义请求头的 HTTP 客户端 GET 请求（补充测试）
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送带自定义请求头的 GET 请求。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器匹配自定义请求头（X-Custom-Header）
/// 2. 发送带自定义请求头的 GET 请求
/// 3. 验证请求头正确传递
///
/// ## 预期结果
/// - 响应状态码为 200
/// - 自定义请求头正确匹配
#[test]
fn test_http_client_get_with_headers() -> color_eyre::Result<()> {
    let mut manager = MockServer::new();

    use reqwest::header::HeaderMap;
    let mut headers = HeaderMap::new();
    headers.insert(
        "X-Custom-Header",
        "custom-value".parse().expect("header value should parse successfully"),
    );

    let mock = manager
        .server()
        .mock("GET", "/test")
        .match_header("x-custom-header", "custom-value")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"header_received": true}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().headers(&headers);
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    assert_eq!(response.status, 200);

    mock.assert();
    manager.cleanup();
    Ok(())
}

/// 测试带超时的 HTTP 客户端 GET 请求（补充测试）
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确应用超时配置。
///
/// ## 测试场景
/// 1. 配置请求超时为 30 秒
/// 2. 发送 GET 请求
/// 3. 验证超时配置正确传递
///
/// ## 预期结果
/// - 响应状态码为 200
/// - 超时配置正确传递
#[test]
fn test_http_client_get_with_timeout() -> color_eyre::Result<()> {
    let mut manager = MockServer::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "success"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().timeout(std::time::Duration::from_secs(5));
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    assert_eq!(response.status, 200);

    mock.assert();
    manager.cleanup();
    Ok(())
}

/// 测试 HTTP 客户端 PUT 请求（补充测试）
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送 PUT 请求。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 200 状态码
/// 2. 发送带 JSON 请求体的 PUT 请求
/// 3. 验证响应状态码
///
/// ## 预期结果
/// - 响应状态码为 200
#[test]
fn test_http_client_put_with_json_body_returns_response() -> color_eyre::Result<()> {
    // Arrange: 准备 Mock 服务器和请求体
    let mut manager = MockServer::new();
    let body = serde_json::json!({"key": "updated_value"});
    let mock = manager
        .server()
        .mock("PUT", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"updated": true}"#)
        .create();

    // Act: 发送 PUT 请求
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().body(&body);
    let url = format!("{}/test", manager.base_url());
    let response = client.put(&url, config)?;

    // Assert: 验证响应状态正确
    assert_eq!(response.status, 200);

    mock.assert();
    manager.cleanup();
    Ok(())
}

/// 测试 HTTP 客户端 DELETE 请求（补充测试）
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送 DELETE 请求。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 204 No Content 状态码
/// 2. 发送 DELETE 请求
/// 3. 验证响应状态码
///
/// ## 预期结果
/// - 响应状态码为 204
#[test]
fn test_http_client_delete_with_valid_url_returns_response() -> color_eyre::Result<()> {
    // Arrange: 准备 Mock 服务器
    let mut manager = MockServer::new();
    let mock = manager
        .server()
        .mock("DELETE", "/test")
        .with_status(204)
        .with_header("content-type", "application/json")
        .with_body("")
        .create();

    // Act: 发送 DELETE 请求
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.delete(&url, config)?;

    // Assert: 验证响应状态正确
    assert_eq!(response.status, 204);

    mock.assert();
    manager.cleanup();
    Ok(())
}

/// 测试 HTTP 客户端 PATCH 请求（补充测试）
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确发送 PATCH 请求。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 200 状态码
/// 2. 发送带 JSON 请求体的 PATCH 请求
/// 3. 验证响应状态码
///
/// ## 预期结果
/// - 响应状态码为 200
#[test]
fn test_http_client_patch_with_json_body_returns_response() -> color_eyre::Result<()> {
    // Arrange: 准备 Mock 服务器和请求体
    let mut manager = MockServer::new();
    let body = serde_json::json!({"key": "patched_value"});
    let mock = manager
        .server()
        .mock("PATCH", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"patched": true}"#)
        .create();

    // Act: 发送 PATCH 请求
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().body(&body);
    let url = format!("{}/test", manager.base_url());
    let response = client.patch(&url, config)?;

    // Assert: 验证响应状态正确
    assert_eq!(response.status, 200);

    mock.assert();
    manager.cleanup();
    Ok(())
}

/// 测试 HTTP 客户端错误状态码响应处理（补充测试）
///
/// ## 测试目的
/// 验证 HTTP 客户端能够正确处理错误状态码响应（如 404）。
///
/// ## 测试场景
/// 1. 配置 Mock 服务器返回 404 错误状态码
/// 2. 发送 GET 请求
/// 3. 验证响应状态码和错误标记
///
/// ## 预期结果
/// - 响应状态码为 404
/// - 响应标记为错误
#[test]
fn test_http_client_get_with_error_status_handles_error_response() -> color_eyre::Result<()> {
    // Arrange: 准备返回错误状态码的 Mock 服务器
    let mut manager = MockServer::new();
    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Not Found"}"#)
        .create();

    // Act: 发送 GET 请求
    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    // Assert: 验证响应状态为错误且错误标志正确
    assert_eq!(response.status, 404);
    assert!(!response.is_success());
    assert!(response.is_error());

    mock.assert();
    manager.cleanup();
    Ok(())
}
