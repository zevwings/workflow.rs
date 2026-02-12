//! HTTP 客户端

use std::time::Duration;

use once_cell::sync::OnceCell;

use reqwest::blocking::Client;

use crate::{config::HttpClientConfig, error::HttpError, method::HttpMethod, request::Request};

/// HTTP 客户端
///
/// 提供 HTTP 请求的封装，支持链式 API 和增强的错误处理。
///
/// # 使用 base_url
///
/// 可以设置 `base_url`，之后请求方法只需传入路径：
///
/// ```rust,ignore
/// let client = HttpClient::with_config(
///     HttpClientConfig::new().base_url("https://api.example.com")
/// )?;
///
/// // 自动拼接为 https://api.example.com/users
/// let response = client.get("/users").send()?;
/// ```
pub struct HttpClient {
    pub(crate) client: Client,
    pub(crate) base_url: Option<String>,
    pub(crate) timeout: Duration,
    pub(crate) max_response_body_size: usize,
}

impl HttpClient {
    /// 获取全局 HttpClient 单例
    pub fn global() -> Result<&'static Self, HttpError> {
        static CLIENT: OnceCell<HttpClient> = OnceCell::new();

        CLIENT.get_or_try_init(|| {
            Self::new().map_err(|e| {
                error!("Failed to create HTTP client: {}", e);
                HttpError::ClientCreation("Failed to create HTTP client".to_string())
            })
        })
    }

    /// 使用默认配置创建客户端
    pub fn new() -> Result<Self, HttpError> {
        Self::with_config(HttpClientConfig::default())
    }

    /// 使用自定义配置创建客户端
    pub fn with_config(config: HttpClientConfig) -> Result<Self, HttpError> {
        let mut builder = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.timeout)
            .user_agent(&config.user_agent);

        if !config.tls_verify {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let client = builder.build().map_err(|e| {
            error!("Failed to create HTTP client: {}", e);
            HttpError::ClientCreation("Failed to build HTTP client".to_string())
        })?;

        Ok(Self {
            client,
            base_url: config.base_url,
            timeout: config.timeout,
            max_response_body_size: config.max_response_body_size,
        })
    }

    /// 解析 URL
    ///
    /// 如果传入的是相对路径（以 `/` 开头），则与 base_url 拼接；
    /// 否则直接返回传入的 URL。
    fn resolve_url(&self, url: &str) -> String {
        if url.starts_with('/') {
            if let Some(base) = &self.base_url {
                return format!("{}{}", base, url);
            }
        }
        url.to_string()
    }

    /// GET 请求
    ///
    /// 如果设置了 `base_url`，可以只传入路径（如 `/users`）。
    pub fn get(&self, url: impl AsRef<str>) -> Request<'_> {
        Request::new(self, HttpMethod::GET, self.resolve_url(url.as_ref()))
    }

    /// POST 请求
    ///
    /// 如果设置了 `base_url`，可以只传入路径（如 `/users`）。
    pub fn post(&self, url: impl AsRef<str>) -> Request<'_> {
        Request::new(self, HttpMethod::POST, self.resolve_url(url.as_ref()))
    }

    /// PUT 请求
    ///
    /// 如果设置了 `base_url`，可以只传入路径（如 `/users/123`）。
    pub fn put(&self, url: impl AsRef<str>) -> Request<'_> {
        Request::new(self, HttpMethod::PUT, self.resolve_url(url.as_ref()))
    }

    /// DELETE 请求
    ///
    /// 如果设置了 `base_url`，可以只传入路径（如 `/users/123`）。
    pub fn delete(&self, url: impl AsRef<str>) -> Request<'_> {
        Request::new(self, HttpMethod::DELETE, self.resolve_url(url.as_ref()))
    }

    /// PATCH 请求
    ///
    /// 如果设置了 `base_url`，可以只传入路径（如 `/users/123`）。
    pub fn patch(&self, url: impl AsRef<str>) -> Request<'_> {
        Request::new(self, HttpMethod::PATCH, self.resolve_url(url.as_ref()))
    }

    /// HEAD 请求
    ///
    /// 如果设置了 `base_url`，可以只传入路径。
    pub fn head(&self, url: impl AsRef<str>) -> Request<'_> {
        Request::new(self, HttpMethod::HEAD, self.resolve_url(url.as_ref()))
    }

    /// OPTIONS 请求
    ///
    /// 如果设置了 `base_url`，可以只传入路径。
    pub fn options(&self, url: impl AsRef<str>) -> Request<'_> {
        Request::new(self, HttpMethod::OPTIONS, self.resolve_url(url.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serial_test::serial;

    use super::*;
    use crate::{auth::Authorization, testing::MockServer, RetryConfig};

    #[test]
    fn test_http_client_new() -> Result<(), HttpError> {
        let client = HttpClient::new()?;
        assert_eq!(client.timeout, Duration::from_secs(30));
        assert!(client.base_url.is_none());

        Ok(())
    }

    #[test]
    fn test_http_client_resolve_url_no_base() -> Result<(), HttpError> {
        let client = HttpClient::new()?;

        // 没有 base_url 时，直接返回传入的 URL
        assert_eq!(client.resolve_url("/users"), "/users");
        assert_eq!(
            client.resolve_url("https://example.com/users"),
            "https://example.com/users"
        );

        Ok(())
    }

    #[test]
    fn test_http_client_resolve_url_with_base() -> Result<(), HttpError> {
        let config = HttpClientConfig::new().base_url("https://api.example.com");
        let client = HttpClient::with_config(config).unwrap();

        // 相对路径会与 base_url 拼接
        assert_eq!(
            client.resolve_url("/users"),
            "https://api.example.com/users"
        );
        assert_eq!(
            client.resolve_url("/users/123"),
            "https://api.example.com/users/123"
        );

        // 完整 URL 不会拼接
        assert_eq!(
            client.resolve_url("https://other.com/data"),
            "https://other.com/data"
        );

        Ok(())
    }

    #[test]
    fn test_http_client_base_url_with_path() -> Result<(), HttpError> {
        let config = HttpClientConfig::new().base_url("https://api.example.com/v1");
        let client = HttpClient::with_config(config)?;

        assert_eq!(
            client.resolve_url("/users"),
            "https://api.example.com/v1/users"
        );

        Ok(())
    }

    #[test]
    fn test_http_client_with_config() -> Result<(), HttpError> {
        let config = HttpClientConfig::new().timeout(Duration::from_secs(60));
        let client = HttpClient::with_config(config)?;
        assert_eq!(client.timeout, Duration::from_secs(60));

        Ok(())
    }

    #[test]
    fn test_http_client_global() -> Result<(), HttpError> {
        let client1 = HttpClient::global()?;
        let client2 = HttpClient::global()?;
        assert!(std::ptr::eq(client1, client2));

        Ok(())
    }

    #[test]
    fn test_http_client_request_methods() -> Result<(), HttpError> {
        let client = HttpClient::new()?;

        let _get = client.get("https://example.com");
        let _post = client.post("https://example.com");
        let _put = client.put("https://example.com");
        let _delete = client.delete("https://example.com");
        let _patch = client.patch("https://example.com");

        Ok(())
    }

    // ==================== 集成测试 ====================

    #[test]
    #[serial]
    fn test_base_url_integration() -> Result<(), HttpError> {
        let mut server = MockServer::new();
        let _mock = server
            .mock("GET", "/api/users")
            .with_status(200)
            .with_body(r#"{"users": []}"#)
            .create();

        // 使用 base_url 配置
        let config = HttpClientConfig::new().base_url(server.url());
        let client = HttpClient::with_config(config)?;

        // 只传入路径
        let response = client.get("/api/users").send()?;

        assert!(response.is_success());
        let json: serde_json::Value = response.json()?;
        assert!(json["users"].is_array());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_get_request() -> Result<(), HttpError> {
        let mut server = MockServer::new();
        let _mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"message": "success"}"#)
            .create();

        let client = HttpClient::new()?;
        let url = format!("{}/test", server.url());

        let response = client.get(&url).send()?;

        assert!(response.is_success());
        assert_eq!(response.status, 200);

        let json: serde_json::Value = response.json()?;
        assert_eq!(json["message"], "success");

        Ok(())
    }

    #[test]
    #[serial]
    fn test_post_request_with_body() -> Result<(), HttpError> {
        let mut server = MockServer::new();
        let _mock = server
            .mock("POST", "/users")
            .with_status(201)
            .with_body(r#"{"id": 123, "name": "test"}"#)
            .create();

        let client = HttpClient::new()?;
        let url = format!("{}/users", server.url());

        let response = client.post(&url).body(&serde_json::json!({"name": "test"})).send()?;

        assert!(response.is_success());
        assert_eq!(response.status, 201);

        let json: serde_json::Value = response.json()?;
        assert_eq!(json["id"], 123);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_request_with_query_params() -> Result<(), HttpError> {
        let mut server = MockServer::new();
        let _mock = server
            .mock("GET", "/search")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("q".into(), "rust".into()),
                mockito::Matcher::UrlEncoded("page".into(), "1".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"results": []}"#)
            .create();

        let client = HttpClient::new()?;
        let url = format!("{}/search", server.url());

        let response =
            client.get(&url).query(&serde_json::json!({"q": "rust", "page": "1"})).send()?;

        assert!(response.is_success());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_request_with_auth() -> Result<(), HttpError> {
        let mut server = MockServer::new();
        let _mock = server
            .mock("GET", "/protected")
            .match_header("authorization", "Bearer test-token")
            .with_status(200)
            .with_body(r#"{"authenticated": true}"#)
            .create();

        let client = HttpClient::new()?;
        let url = format!("{}/protected", server.url());

        let response = client.get(&url).auth(Authorization::bearer("test-token")).send().unwrap();

        assert!(response.is_success());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_request_with_custom_headers() -> Result<(), HttpError> {
        let mut server = MockServer::new();
        let _mock = server
            .mock("GET", "/test")
            .match_header("x-custom-header", "custom-value")
            .with_status(200)
            .create();

        let client = HttpClient::new()?;
        let url = format!("{}/test", server.url());

        let response = client.get(&url).header("X-Custom-Header", "custom-value").send()?;

        assert!(response.is_success());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_error_response() -> Result<(), HttpError> {
        let mut server = MockServer::new();
        let _mock = server
            .mock("GET", "/error")
            .with_status(500)
            .with_body(r#"{"error": "Internal Server Error"}"#)
            .create();

        let client = HttpClient::new()?;
        let url = format!("{}/error", server.url());

        let response = client.get(&url).send()?;

        assert!(!response.is_success());
        assert!(response.is_server_error());
        assert_eq!(response.status, 500);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_ensure_success() -> Result<(), HttpError> {
        let mut server = MockServer::new();
        let _mock = server.mock("GET", "/error").with_status(404).create();

        let client = HttpClient::new()?;
        let url = format!("{}/error", server.url());

        let response = client.get(&url).send()?;
        let result = response.ensure_success();

        assert!(result.is_err());
        if let Err(HttpError::Status { status, .. }) = result {
            assert_eq!(status, 404);
        } else {
            panic!("Expected Status error");
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_extract_error_message() -> Result<(), HttpError> {
        let mut server = MockServer::new();
        let _mock = server
            .mock("GET", "/error")
            .with_status(400)
            .with_body(r#"{"error": "Bad request"}"#)
            .create();

        let client = HttpClient::new()?;
        let url = format!("{}/error", server.url());

        let response = client.get(&url).send()?;
        let error_msg = response.extract_error_message();

        assert_eq!(error_msg, "Bad request");

        Ok(())
    }

    #[test]
    #[serial]
    fn test_put_request() -> Result<(), HttpError> {
        let mut server = MockServer::new();
        let _mock = server
            .mock("PUT", "/users/123")
            .with_status(200)
            .with_body(r#"{"updated": true}"#)
            .create();

        let client = HttpClient::new()?;
        let url = format!("{}/users/123", server.url());

        let response =
            client.put(&url).body(&serde_json::json!({"name": "updated"})).send().unwrap();

        assert!(response.is_success());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_delete_request() -> Result<(), HttpError> {
        let mut server = MockServer::new();
        let _mock = server.mock("DELETE", "/users/123").with_status(204).create();

        let client = HttpClient::new()?;
        let url = format!("{}/users/123", server.url());

        let response = client.delete(&url).send()?;

        assert!(response.is_success());
        assert_eq!(response.status, 204);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_patch_request() -> Result<(), HttpError> {
        let mut server = MockServer::new();
        let _mock = server
            .mock("PATCH", "/users/123")
            .with_status(200)
            .with_body(r#"{"patched": true}"#)
            .create();

        let client = HttpClient::new()?;
        let url = format!("{}/users/123", server.url());

        let response = client.patch(&url).body(&serde_json::json!({"field": "value"})).send()?;

        assert!(response.is_success());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_request_with_retry() -> Result<(), HttpError> {
        let mut server = MockServer::new();
        let _mock1 = server.mock("GET", "/retry").with_status(500).expect_at_most(2).create();
        let _mock2 = server
            .mock("GET", "/retry")
            .with_status(200)
            .with_body(r#"{"success": true}"#)
            .create();

        let client = HttpClient::new()?;
        let url = format!("{}/retry", server.url());

        let result = client
            .get(&url)
            .retry(RetryConfig::new().max_retries(3).initial_delay(Duration::from_millis(10)))
            .send()?;

        assert!(result.is_success());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_json_shortcut() -> Result<(), HttpError> {
        let mut server = MockServer::new();
        let _mock = server
            .mock("GET", "/data")
            .with_status(200)
            .with_body(r#"{"id": 42, "name": "test"}"#)
            .create();

        let client = HttpClient::new()?;
        let url = format!("{}/data", server.url());

        let data: serde_json::Value = client.get(&url).json()?;

        assert_eq!(data["id"], 42);
        assert_eq!(data["name"], "test");

        Ok(())
    }

    #[test]
    #[serial]
    fn test_text_shortcut() -> Result<(), HttpError> {
        let mut server = MockServer::new();
        let _mock =
            server.mock("GET", "/text").with_status(200).with_body("Hello, World!").create();

        let client = HttpClient::new()?;
        let url = format!("{}/text", server.url());

        let text = client.get(&url).text()?;

        assert_eq!(text, "Hello, World!");

        Ok(())
    }
}
