//! Base HTTP Client 模块测试
//!
//! 测试 HTTP 客户端的核心功能，包括 HttpClient 结构体。
//! 使用 mock server 来测试实际的 HTTP 请求。

use pretty_assertions::assert_eq;
use serde_json::Value;
use workflow::base::http::{Authorization, HttpClient, RequestConfig};
use crate::common::mock_server::MockServerManager;

#[test]
fn test_http_client_global() -> color_eyre::Result<()> {
    // 测试可以获取全局 HttpClient 单例
    let _client = HttpClient::global()?;
    // 验证客户端可以获取
    assert!(true);
    Ok(())
}

#[test]
fn test_http_client_get() -> color_eyre::Result<()> {
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
    let json: Value = response.as_json()?;
    assert_eq!(json["message"], "success");

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_client_post() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

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
    let mut manager = MockServerManager::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .match_header("authorization", mockito::Matcher::Regex(r"Basic .+".to_string()))
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
    let mut manager = MockServerManager::new();

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
    let mut manager = MockServerManager::new();

    use reqwest::header::HeaderMap;
    let mut headers = HeaderMap::new();
    headers.insert("X-Custom-Header", "custom-value".parse().unwrap());

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
    let mut manager = MockServerManager::new();

    let mock = manager
        .server()
        .mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "success"}"#)
        .create();

    let client = HttpClient::global()?;
    let config = RequestConfig::<Value, Value>::new()
        .timeout(std::time::Duration::from_secs(5));
    let url = format!("{}/test", manager.base_url());
    let response = client.get(&url, config)?;

    assert_eq!(response.status, 200);

    mock.assert();
    manager.cleanup();
    Ok(())
}

#[test]
fn test_http_client_put() -> color_eyre::Result<()> {
    let mut manager = MockServerManager::new();

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
    let mut manager = MockServerManager::new();

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
    let mut manager = MockServerManager::new();

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
