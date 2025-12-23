//! HTTP Config 测试
//!
//! 测试 HTTP 请求配置和 Multipart 请求配置的功能。

use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use workflow::base::http::{Authorization, MultipartRequestConfig, RequestConfig};

#[test]
fn test_request_config_new() {
    let config = RequestConfig::<Value, Value>::new();
    assert!(config.body.is_none());
    assert!(config.query.is_none());
    assert!(config.auth.is_none());
    assert!(config.headers.is_none());
    assert!(config.timeout.is_none());
}

#[test]
fn test_request_config_default() {
    let config = RequestConfig::<Value, Value>::default();
    assert!(config.body.is_none());
    assert!(config.query.is_none());
    assert!(config.auth.is_none());
    assert!(config.headers.is_none());
    assert!(config.timeout.is_none());
}

#[test]
fn test_request_config_body() {
    let body = serde_json::json!({"key": "value"});
    let config = RequestConfig::<Value, Value>::new().body(&body);
    assert!(config.body.is_some());
    assert_eq!(config.body.unwrap()["key"], "value");
}

#[test]
fn test_request_config_query() {
    let query = [("page", "1"), ("limit", "10")];
    let config = RequestConfig::<Value, _>::new().query(&query);
    assert!(config.query.is_some());
}

#[test]
fn test_request_config_query_hashmap() {
    let mut params = HashMap::new();
    params.insert("state", "open");
    params.insert("sort", "created");
    let config = RequestConfig::<Value, _>::new().query(&params);
    assert!(config.query.is_some());
}

#[test]
fn test_request_config_auth() {
    let auth = Authorization::new("username", "password");
    let config = RequestConfig::<Value, Value>::new().auth(&auth);
    assert!(config.auth.is_some());
    let config_auth = config.auth.unwrap();
    assert_eq!(config_auth.username, "username");
    assert_eq!(config_auth.password, "password");
}

#[test]
fn test_request_config_headers() {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-Custom-Header", "value".parse().unwrap());
    let config = RequestConfig::<Value, Value>::new().headers(&headers);
    assert!(config.headers.is_some());
    assert_eq!(
        config.headers.unwrap().get("X-Custom-Header").unwrap(),
        "value"
    );
}

#[test]
fn test_request_config_timeout() {
    let timeout = Duration::from_secs(60);
    let config = RequestConfig::<Value, Value>::new().timeout(timeout);
    assert!(config.timeout.is_some());
    assert_eq!(config.timeout.unwrap(), timeout);
}

#[test]
fn test_request_config_chain() {
    let body = serde_json::json!({"data": "test"});
    let query = [("filter", "active")];
    let auth = Authorization::new("user", "pass");
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-Request-ID", "12345".parse().unwrap());
    let timeout = Duration::from_secs(30);

    let config = RequestConfig::new()
        .body(&body)
        .query(&query)
        .auth(&auth)
        .headers(&headers)
        .timeout(timeout);

    assert!(config.body.is_some());
    assert!(config.query.is_some());
    assert!(config.auth.is_some());
    assert!(config.headers.is_some());
    assert!(config.timeout.is_some());
}

#[test]
fn test_multipart_request_config_new() {
    let config = MultipartRequestConfig::<Value>::new();
    assert!(config.multipart.is_none());
    assert!(config.query.is_none());
    assert!(config.auth.is_none());
    assert!(config.headers.is_none());
    assert!(config.timeout.is_none());
}

#[test]
fn test_multipart_request_config_default() {
    let config = MultipartRequestConfig::<Value>::default();
    assert!(config.multipart.is_none());
    assert!(config.query.is_none());
    assert!(config.auth.is_none());
    assert!(config.headers.is_none());
    assert!(config.timeout.is_none());
}

#[test]
fn test_multipart_request_config_multipart() {
    let form = reqwest::blocking::multipart::Form::new()
        .text("key", "value")
        .text("name", "test");
    let config = MultipartRequestConfig::<Value>::new().multipart(form);
    assert!(config.multipart.is_some());
}

#[test]
fn test_multipart_request_config_query() {
    let query = serde_json::json!({"param": "value"});
    let config = MultipartRequestConfig::new().query(query);
    assert!(config.query.is_some());
}

#[test]
fn test_multipart_request_config_auth() {
    let auth = Authorization::new("username", "password");
    let auth_clone = auth.clone();
    let config = MultipartRequestConfig::<Value>::new().auth(auth);
    assert!(config.auth.is_some());
    let config_auth = config.auth.unwrap();
    assert_eq!(config_auth.username, auth_clone.username);
    assert_eq!(config_auth.password, auth_clone.password);
}

#[test]
fn test_multipart_request_config_headers() {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-Custom-Header", "value".parse().unwrap());
    let config = MultipartRequestConfig::<Value>::new().headers(headers.clone());
    assert!(config.headers.is_some());
    assert_eq!(
        config.headers.unwrap().get("X-Custom-Header").unwrap(),
        "value"
    );
}

#[test]
fn test_multipart_request_config_timeout() {
    let timeout = Duration::from_secs(120);
    let config = MultipartRequestConfig::<Value>::new().timeout(timeout);
    assert!(config.timeout.is_some());
    assert_eq!(config.timeout.unwrap(), timeout);
}

#[test]
fn test_multipart_request_config_chain() {
    let form = reqwest::blocking::multipart::Form::new().text("file", "content");
    let query = serde_json::json!({"param": "value"});
    let auth = Authorization::new("user", "pass");
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-Upload-ID", "67890".parse().unwrap());
    let timeout = Duration::from_secs(60);

    let config = MultipartRequestConfig::new()
        .multipart(form)
        .query(query)
        .auth(auth)
        .headers(headers)
        .timeout(timeout);

    assert!(config.multipart.is_some());
    assert!(config.query.is_some());
    assert!(config.auth.is_some());
    assert!(config.headers.is_some());
    assert!(config.timeout.is_some());
}
