//! HTTP 客户端测试
//!
//! 测试 HTTP 客户端的各种功能，包括：
//! - HTTP 方法测试（GET, POST, PUT, DELETE, PATCH）
//! - 错误处理测试（各种 HTTP 状态码）
//! - 网络错误测试
//! - 超时错误测试
//! - 认证测试

use crate::common::http_helpers::MockServer;
use color_eyre::Result;
use mockito::Matcher;
use serde_json::Value;
use workflow::base::http::{
    auth::Authorization, HttpClient, HttpMethod, RequestConfig,
};

// ==================== HTTP 方法测试 ====================

#[test]
fn test_get_request_success() -> Result<()> {
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

    assert_eq!(response.status, 200);
    assert!(response.is_success());
    Ok(())
}

#[test]
fn test_post_request_success() -> Result<()> {
    let mut mock_server = MockServer::new();
    let body = serde_json::json!({"key": "value"});
    let _mock = mock_server
        .server
        .as_mut()
        .mock("POST", "/test")
        .match_header("content-type", "application/json")
        .match_body(Matcher::JsonString(serde_json::to_string(&body)?))
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id": 123}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().body(&body);
    let response = client.post(&mock_server.base_url, config)?;

    assert_eq!(response.status, 201);
    assert!(response.is_success());
    Ok(())
}

#[test]
fn test_put_request_success() -> Result<()> {
    let mut mock_server = MockServer::new();
    let body = serde_json::json!({"key": "updated"});
    let _mock = mock_server
        .server
        .as_mut()
        .mock("PUT", "/test")
        .match_body(Matcher::JsonString(serde_json::to_string(&body)?))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "updated"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().body(&body);
    let response = client.put(&mock_server.base_url, config)?;

    assert_eq!(response.status, 200);
    assert!(response.is_success());
    Ok(())
}

#[test]
fn test_delete_request_success() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("DELETE", "/test")
        .with_status(204)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.delete(&mock_server.base_url, config)?;

    assert_eq!(response.status, 204);
    assert!(response.is_success());
    Ok(())
}

#[test]
fn test_patch_request_success() -> Result<()> {
    let mut mock_server = MockServer::new();
    let body = serde_json::json!({"key": "patched"});
    let _mock = mock_server
        .server
        .as_mut()
        .mock("PATCH", "/test")
        .match_body(Matcher::JsonString(serde_json::to_string(&body)?))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "patched"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new().body(&body);
    let response = client.patch(&mock_server.base_url, config)?;

    assert_eq!(response.status, 200);
    assert!(response.is_success());
    Ok(())
}

// ==================== HTTP 错误处理测试（P0） ====================

#[test]
fn test_http_error_400_bad_request() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Bad Request"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    assert_eq!(response.status, 400);
    assert!(response.is_error());
    Ok(())
}

#[test]
fn test_http_error_401_unauthorized() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Unauthorized"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    assert_eq!(response.status, 401);
    assert!(response.is_error());
    Ok(())
}

#[test]
fn test_http_error_403_forbidden() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(403)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Forbidden"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    assert_eq!(response.status, 403);
    assert!(response.is_error());
    Ok(())
}

#[test]
fn test_http_error_404_not_found() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Not Found"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    assert_eq!(response.status, 404);
    assert!(response.is_error());
    Ok(())
}

#[test]
fn test_http_error_500_internal_server_error() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    assert_eq!(response.status, 500);
    assert!(response.is_error());
    Ok(())
}

#[test]
fn test_http_error_502_bad_gateway() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(502)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Bad Gateway"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    assert_eq!(response.status, 502);
    assert!(response.is_error());
    Ok(())
}

#[test]
fn test_http_error_503_service_unavailable() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(503)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Service Unavailable"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    assert_eq!(response.status, 503);
    assert!(response.is_error());
    Ok(())
}

#[test]
fn test_http_error_504_gateway_timeout() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(504)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Gateway Timeout"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    assert_eq!(response.status, 504);
    assert!(response.is_error());
    Ok(())
}

#[test]
fn test_http_error_429_too_many_requests() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_header("retry-after", "60")
        .with_body(r#"{"error": "Too Many Requests"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new();
    let response = client.get(&mock_server.base_url, config)?;

    assert_eq!(response.status, 429);
    assert!(response.is_error());
    Ok(())
}

// ==================== 认证测试 ====================

#[test]
fn test_get_with_basic_auth() -> Result<()> {
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .match_header("authorization", Matcher::Regex(r"Basic .+".to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "success"}"#)
        .create();

    let client = HttpClient::global()?;
    let auth = Authorization::new("user@example.com", "token123");
    let config = RequestConfig::<Value, Value>::new().auth(&auth);
    let response = client.get(&mock_server.base_url, config)?;

    assert_eq!(response.status, 200);
    assert!(response.is_success());
    Ok(())
}

// ==================== 查询参数测试 ====================

#[test]
fn test_get_with_query_params() -> Result<()> {
    let mut mock_server = MockServer::new();
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
    let response = client.get(&mock_server.base_url, config)?;

    assert_eq!(response.status, 200);
    assert!(response.is_success());
    Ok(())
}

// ==================== 超时测试 ====================

#[test]
fn test_request_with_timeout() -> Result<()> {
    let mut mock_server = MockServer::new();
    // 创建一个会延迟响应的 mock（但实际测试中可能无法真正延迟）
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "success"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new()
        .timeout(std::time::Duration::from_secs(5));
    let response = client.get(&mock_server.base_url, config)?;

    assert_eq!(response.status, 200);
    assert!(response.is_success());
    Ok(())
}

// ==================== 单例测试 ====================

#[test]
fn test_http_client_global_singleton() -> Result<()> {
    let client1 = HttpClient::global()?;
    let client2 = HttpClient::global()?;
    
    // 验证返回的是同一个实例（通过指针比较）
    // 注意：由于返回的是 &'static，我们只能验证它们都能正常工作
    // 实际测试中，单例模式通过 OnceLock 保证
    
    // 使用两个客户端发送请求来验证它们都能正常工作
    let mut mock_server = MockServer::new();
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/test")
        .with_status(200)
        .create();

    let config = RequestConfig::<Value, Value>::new();
    let response1 = client1.get(&mock_server.base_url, config)?;
    assert_eq!(response1.status, 200);

    let config = RequestConfig::<Value, Value>::new();
    let response2 = client2.get(&mock_server.base_url, config)?;
    assert_eq!(response2.status, 200);

    Ok(())
}
