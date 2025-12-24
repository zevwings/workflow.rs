//! Base HTTP Config 模块测试
//!
//! 测试 HTTP 请求配置的核心功能，包括 RequestConfig 和 MultipartRequestConfig。

use pretty_assertions::assert_eq;
use reqwest::header::HeaderMap;
use serde_json::Value;
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
    assert_eq!(config.body.unwrap(), &body);
}

#[test]
fn test_request_config_query() {
    let query = [("page", "1"), ("per_page", "10")];
    let config = RequestConfig::<Value, _>::new().query(&query);
    assert!(config.query.is_some());
}

#[test]
fn test_request_config_auth() {
    let auth = Authorization::new("user@example.com", "token");
    let config = RequestConfig::<Value, Value>::new().auth(&auth);
    assert!(config.auth.is_some());
    assert_eq!(config.auth.unwrap().username, "user@example.com");
}

#[test]
fn test_request_config_headers() {
    let mut headers = HeaderMap::new();
    headers.insert("X-Custom-Header", "value".parse().unwrap());
    let config = RequestConfig::<Value, Value>::new().headers(&headers);
    assert!(config.headers.is_some());
}

#[test]
fn test_request_config_timeout() {
    let timeout = Duration::from_secs(60);
    let config = RequestConfig::<Value, Value>::new().timeout(timeout);
    assert_eq!(config.timeout, Some(timeout));
}

#[test]
fn test_request_config_chain() {
    let body = serde_json::json!({"key": "value"});
    let auth = Authorization::new("user@example.com", "token");
    let timeout = Duration::from_secs(30);

    let config = RequestConfig::<Value, Value>::new()
        .body(&body)
        .auth(&auth)
        .timeout(timeout);

    assert!(config.body.is_some());
    assert!(config.auth.is_some());
    assert_eq!(config.timeout, Some(timeout));
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
    use reqwest::blocking::multipart;
    let form = multipart::Form::new();
    let config = MultipartRequestConfig::<Value>::new().multipart(form);
    assert!(config.multipart.is_some());
}

#[test]
fn test_multipart_request_config_query() {
    let query = serde_json::json!({"param": "value"});
    let config = MultipartRequestConfig::new().query(query.clone());
    assert_eq!(config.query, Some(query));
}

#[test]
fn test_multipart_request_config_auth() {
    let auth = Authorization::new("user@example.com", "token");
    let config = MultipartRequestConfig::<Value>::new().auth(auth.clone());
    assert!(config.auth.is_some());
    assert_eq!(config.auth.as_ref().unwrap().username, auth.username);
    assert_eq!(config.auth.as_ref().unwrap().password, auth.password);
}

#[test]
fn test_multipart_request_config_headers() {
    let mut headers = HeaderMap::new();
    headers.insert("X-Custom-Header", "value".parse().unwrap());
    let config = MultipartRequestConfig::<Value>::new().headers(headers.clone());
    assert_eq!(config.headers, Some(headers));
}

#[test]
fn test_multipart_request_config_timeout() {
    let timeout = Duration::from_secs(60);
    let config = MultipartRequestConfig::<Value>::new().timeout(timeout);
    assert_eq!(config.timeout, Some(timeout));
}

#[test]
fn test_multipart_request_config_chain() {
    use reqwest::blocking::multipart;
    let form = multipart::Form::new();
    let auth = Authorization::new("user@example.com", "token");
    let timeout = Duration::from_secs(30);

    let config = MultipartRequestConfig::<Value>::new()
        .multipart(form)
        .auth(auth.clone())
        .timeout(timeout);

    assert!(config.multipart.is_some());
    assert!(config.auth.is_some());
    assert_eq!(config.auth.as_ref().unwrap().username, auth.username);
    assert_eq!(config.auth.as_ref().unwrap().password, auth.password);
    assert_eq!(config.timeout, Some(timeout));
}

