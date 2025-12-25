//! HTTP Client 测试
//!
//! 测试 HTTP 客户端的各种功能，包括 GET、POST、PUT、DELETE、PATCH 等方法。
//!
//! ## 测试策略
//!
//! - 使用 MockServer 模拟 HTTP 服务器，避免实际网络请求
//! - 所有测试返回 Result<()>，使用 ? 运算符处理错误
//! - 测试覆盖成功场景、错误场景、边界条件

use crate::common::http_helpers::MockServer;
use color_eyre::Result;
use mockito::Matcher;
use serde_json::Value;
use workflow::base::http::{Authorization, HttpClient, RequestConfig};

/// 设置 Mock 服务器
fn setup_mock_server() -> MockServer {
    MockServer::new()
}

#[test]
fn test_http_client_global() -> Result<()> {
    // 测试全局单例创建
    let _client1 = HttpClient::global()?;

    // 测试单例复用
    let _client2 = HttpClient::global()?;

    // 验证都能成功获取客户端实例
    Ok(())
}

#[test]
fn test_get_request_success() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    assert_eq!(response.status, 200);
    assert!(response.is_success());

    _mock.assert();
    Ok(())
}

#[test]
fn test_get_request_with_query() -> Result<()> {
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

    let client = HttpClient::global()?;
    let query = [("page", "1"), ("limit", "10")];
    let config = RequestConfig::<Value, _>::new().query(&query);
    let response = client.get(&url, config)?;

    assert!(response.is_success());
    _mock.assert();
    Ok(())
}

#[test]
fn test_get_request_with_auth() -> Result<()> {
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

    let client = HttpClient::global()?;
    let auth = workflow::base::http::Authorization::new("user", "pass");
    let config = RequestConfig::<Value, Value>::new().auth(&auth);
    let response = client.get(&url, config)?;

    assert!(response.is_success());
    _mock.assert();
    Ok(())
}

#[test]
fn test_get_request_with_headers() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().headers(&headers);
    let response = client.get(&url, config)?;

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
#[test]
fn test_post_request_success() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().body(&body_data);
    let response = client.post(&url, config)?;

    assert_eq!(response.status, 201);
    _mock.assert();
    Ok(())
}

#[test]
fn test_put_request_success() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().body(&body_data);
    let response = client.put(&url, config)?;

    assert_eq!(response.status, 200);
    _mock.assert();
    Ok(())
}

#[test]
fn test_delete_request_success() -> Result<()> {
    let mut mock_server = setup_mock_server();
    let url = format!("{}/test/1", mock_server.base_url);

    let _mock = mock_server.server.as_mut().mock("DELETE", "/test/1").with_status(204).create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.delete(&url, config)?;

    assert_eq!(response.status, 204);
    _mock.assert();
    Ok(())
}

#[test]
fn test_patch_request_success() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().body(&body_data);
    let response = client.patch(&url, config)?;

    assert_eq!(response.status, 200);
    _mock.assert();
    Ok(())
}

#[test]
fn test_post_multipart_request() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = workflow::base::http::MultipartRequestConfig::<Value>::new().multipart(form);
    let response = client.post_multipart(&url, config)?;

    assert!(response.is_success());
    _mock.assert();
    Ok(())
}

#[test]
fn test_stream_request() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let mut stream = client.stream(workflow::base::http::HttpMethod::Get, &url, config)?;

    let mut buffer = Vec::new();
    stream.copy_to(&mut buffer)?;
    assert_eq!(buffer, b"streaming data");
    _mock.assert();
    Ok(())
}

#[test]
fn test_request_timeout() -> Result<()> {
    let mut mock_server = setup_mock_server();
    let url = format!("{}/slow", mock_server.base_url);

    // 注意：mockito 不支持真正的延迟，这里只测试配置是否正确传递
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/slow")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().timeout(std::time::Duration::from_secs(5));
    let response = client.get(&url, config)?;

    assert!(response.is_success());
    _mock.assert();
    Ok(())
}

#[test]
fn test_request_error_handling() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&url, config)?;

    // 请求应该成功（HTTP 客户端层面），但响应状态码是 500
    assert_eq!(response.status, 500);
    assert!(response.is_error());
    _mock.assert();
    Ok(())
}

#[test]
fn test_request_with_all_options() -> Result<()> {
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

    let client = HttpClient::global()?;
    let config = RequestConfig::new()
        .body(&body_data)
        .query(&query)
        .auth(&auth)
        .headers(&headers)
        .timeout(std::time::Duration::from_secs(10));
    let response = client.post(&url, config)?;

    assert!(response.is_success());
    _mock.assert();
    Ok(())
}

// ==================== 来自 client_core.rs 的补充测试 ====================

#[test]
fn test_http_client_get() -> color_eyre::Result<()> {
    let mut manager = MockServer::new();

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
    let json: Value = response.as_json()?;
    assert_eq!(json["message"], "success");

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_client_post() -> color_eyre::Result<()> {
    let mut manager = MockServer::new();

    let body = serde_json::json!({"key": "value"});
    let mock = manager
        .server()
        .mock("POST", "/test")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id": 123, "key": "value"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().body(&body);
    let url = format!("{}/test", manager.base_url());
    let response = client.post(&url, config)?;

    assert_eq!(response.status, 201);
    let json: Value = response.as_json()?;
    assert_eq!(json["id"], 123);

    mock.assert();
    manager.cleanup();
    Ok(())
}

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

#[test]
fn test_http_client_put() -> color_eyre::Result<()> {
    let mut manager = MockServer::new();

    let body = serde_json::json!({"key": "updated_value"});
    let mock = manager
        .server()
        .mock("PUT", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"updated": true}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().body(&body);
    let url = format!("{}/test", manager.base_url());
    let response = client.put(&url, config)?;

    assert_eq!(response.status, 200);

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_client_delete() -> color_eyre::Result<()> {
    let mut manager = MockServer::new();

    let mock = manager
        .server()
        .mock("DELETE", "/test")
        .with_status(204)
        .with_header("content-type", "application/json")
        .with_body("")
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let url = format!("{}/test", manager.base_url());
    let response = client.delete(&url, config)?;

    assert_eq!(response.status, 204);

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_client_patch() -> color_eyre::Result<()> {
    let mut manager = MockServer::new();

    let body = serde_json::json!({"key": "patched_value"});
    let mock = manager
        .server()
        .mock("PATCH", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"patched": true}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().body(&body);
    let url = format!("{}/test", manager.base_url());
    let response = client.patch(&url, config)?;

    assert_eq!(response.status, 200);

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_client_error_response() -> color_eyre::Result<()> {
    let mut manager = MockServer::new();

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
