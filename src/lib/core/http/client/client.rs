//! HTTP 客户端实现

use crate::core::http::client::config::HttpClientConfig;
use crate::core::http::client::error::ClientHttpError;
use crate::core::http::client::helpers::{apply_auth_for_multipart, apply_common_config};
use crate::core::http::client::method::HttpMethod;
use crate::core::http::request::{MultipartRequestConfig, RequestConfig};
use crate::core::http::{HttpResponse, HttpRetry, HttpRetryConfig};
use color_eyre::Result;
use reqwest::blocking::Client;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// HTTP 客户端
///
/// 提供 HTTP 请求的封装，支持 GET、POST、PUT、DELETE、PATCH 等方法。
/// 支持 Basic Authentication 和自定义 Headers。
pub struct HttpClient {
    /// 内部的 reqwest 客户端
    client: Client,
    /// HTTP 客户端配置
    config: HttpClientConfig,
}

impl HttpClient {
    /// 获取全局 HttpClient 单例
    ///
    /// 返回进程级别的 HttpClient 单例，使用默认配置。
    /// 单例会在首次调用时初始化，后续调用会复用同一个实例。
    ///
    /// # 返回
    ///
    /// 返回 `HttpClient` 的静态引用。
    ///
    /// # 错误
    ///
    /// 如果创建客户端失败，返回相应的错误信息。
    ///
    /// # 优势
    ///
    /// - 复用连接池：所有请求共享同一个连接池，提高性能
    /// - 减少资源消耗：避免重复创建客户端实例
    /// - 线程安全：可以在多线程环境中安全使用
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::{HttpClient, RequestConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = HttpClient::global()?;
    /// let config = RequestConfig::new();
    /// let response = client.get("https://api.example.com", config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn global() -> Result<&'static Self> {
        static CLIENT: OnceLock<Result<HttpClient>> = OnceLock::new();
        match CLIENT.get_or_init(HttpClient::new) {
            Ok(client) => Ok(client),
            Err(e) => {
                let error_msg = e.to_string();
                Err(color_eyre::eyre::eyre!("{}", error_msg))
            }
        }
    }

    /// 创建新的 HttpClient（私有方法）
    ///
    /// 初始化 HTTP 客户端，使用默认配置。
    /// 此方法仅在 `global()` 方法内部使用，用于初始化全局单例。
    ///
    /// # 返回
    ///
    /// 返回 `HttpClient` 结构体。
    ///
    /// # 错误
    ///
    /// 如果创建客户端失败，返回相应的错误信息。
    fn new() -> Result<Self> {
        let config = HttpClientConfig::default();
        Self::with_config(config)
    }

    /// 使用指定配置创建新的 HttpClient
    ///
    /// # 参数
    ///
    /// * `config` - HTTP 客户端配置
    ///
    /// # 返回
    ///
    /// 返回 `HttpClient` 结构体。
    ///
    /// # 错误
    ///
    /// 如果创建客户端失败，返回相应的错误信息。
    fn with_config(config: HttpClientConfig) -> Result<Self> {
        let mut builder = Client::builder()
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .tcp_keepalive(config.keep_alive_timeout)
            .connect_timeout(config.connect_timeout)
            .timeout(config.timeout)
            .user_agent(&config.user_agent);

        if !config.tls_verify {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let client = builder.build().map_err(ClientHttpError::CreateClientFailed)?;
        Ok(Self { client, config })
    }

    /// GET 请求
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::{HttpClient, RequestConfig, HttpRetryConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = HttpClient::global()?;
    /// let query = serde_json::json!({"page": "1"});
    /// let config = RequestConfig::new()
    ///     .query(&query)
    ///     .retry(HttpRetryConfig::new());
    /// let response = client.get("https://api.example.com", config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&self, url: &str, config: RequestConfig) -> Result<HttpResponse> {
        self.execute_request(HttpMethod::Get, url, config)
    }

    /// POST 请求
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::{HttpClient, RequestConfig, HttpRetryConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = HttpClient::global()?;
    /// let body = serde_json::json!({"key": "value"});
    /// let config = RequestConfig::new()
    ///     .body(&body)
    ///     .retry(HttpRetryConfig::new());
    /// let response = client.post("https://api.example.com", config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn post(&self, url: &str, config: RequestConfig) -> Result<HttpResponse> {
        self.execute_request(HttpMethod::Post, url, config)
    }

    /// PUT 请求
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::{HttpClient, RequestConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = HttpClient::global()?;
    /// let body = serde_json::json!({"key": "value"});
    /// let config = RequestConfig::new().body(&body);
    /// let response = client.put("https://api.example.com", config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn put(&self, url: &str, config: RequestConfig) -> Result<HttpResponse> {
        self.execute_request(HttpMethod::Put, url, config)
    }

    /// DELETE 请求
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::{HttpClient, RequestConfig, HttpRetryConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = HttpClient::global()?;
    /// let config = RequestConfig::new()
    ///     .retry(HttpRetryConfig::new());
    /// let response = client.delete("https://api.example.com", config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn delete(&self, url: &str, config: RequestConfig) -> Result<HttpResponse> {
        self.execute_request(HttpMethod::Delete, url, config)
    }

    /// PATCH 请求
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::{HttpClient, RequestConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = HttpClient::global()?;
    /// let body = serde_json::json!({"key": "value"});
    /// let config = RequestConfig::new().body(&body);
    /// let response = client.patch("https://api.example.com", config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn patch(&self, url: &str, config: RequestConfig) -> Result<HttpResponse> {
        self.execute_request(HttpMethod::Patch, url, config)
    }

    /// 流式请求
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::{HttpClient, HttpMethod, RequestConfig};
    /// use std::io::Read;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = HttpClient::global()?;
    /// let query = serde_json::json!({"page": "1"});
    /// let config = RequestConfig::new()
    ///     .query(&query);
    /// let mut response = client.stream(HttpMethod::Get, "https://example.com/api", config)?;
    /// let mut buffer = vec![0u8; 8192];
    /// response.read(&mut buffer)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn stream(
        &self,
        method: HttpMethod,
        url: &str,
        mut config: RequestConfig,
    ) -> Result<reqwest::blocking::Response> {
        let method_str = method.to_string();
        let retry_config = config.retry_config.take();
        let operation_name = format!("{} {}", method_str, url);

        let mut request_config = config.clone();
        request_config.retry_config = None;

        Self::execute_with_retry(
            || {
                let span = tracing::span!(
                    tracing::Level::DEBUG,
                    "http.client.stream",
                    module = "http",
                    http.method = %method_str,
                    http.url = %url,
                );
                let _guard = span.enter();
                let start = Instant::now();

                self.build_request(method, url, request_config.clone()).send().map_err(|e| {
                    let duration = start.elapsed();
                    let error = Self::handle_request_error(e, url, &method_str);
                    tracing::error!(
                        module = "http",
                        http.method = %method_str,
                        http.url = %url,
                        http.duration_ms = duration.as_millis(),
                        error = %error,
                        "HTTP stream request failed"
                    );
                    error.into()
                })
            },
            retry_config.as_ref(),
            &operation_name,
        )
    }

    /// POST Multipart 请求
    ///
    /// 发送 multipart/form-data 请求，通常用于文件上传。
    ///
    /// # 参数
    ///
    /// * `url` - 请求 URL
    /// * `config` - Multipart 请求配置
    ///
    /// # 返回
    ///
    /// 返回 `HttpResponse`。
    ///
    /// # 错误
    ///
    /// 如果请求失败，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::{HttpClient, MultipartRequestConfig};
    /// use reqwest::blocking::multipart;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = HttpClient::global()?;
    /// let form = multipart::Form::new();
    /// let config = MultipartRequestConfig::new()
    ///     .multipart(form);
    /// let response = client.post_multipart("https://api.example.com/upload", config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn post_multipart(
        &self,
        url: &str,
        mut config: MultipartRequestConfig,
    ) -> Result<HttpResponse> {
        if config.retry_config.is_some() {
            tracing::warn!(
                module = "http",
                "Retry config is ignored for multipart requests. Use HttpRetry::retry externally if retry is needed."
            );
        }
        config.retry_config = None;

        let span = tracing::span!(
            tracing::Level::DEBUG,
            "http.client.request",
            module = "http",
            http.method = "POST",
            http.url = %url,
            http.content_type = "multipart/form-data",
        );
        let _guard = span.enter();
        let start = Instant::now();

        let response = self.build_multipart_request(url, config).send().map_err(|e| {
            let duration = start.elapsed();
            let error = Self::handle_request_error(e, url, "POST multipart");
            tracing::error!(
                module = "http",
                http.method = "POST",
                http.url = %url,
                http.content_type = "multipart/form-data",
                http.duration_ms = duration.as_millis(),
                error = %error,
                "HTTP multipart request failed"
            );
            error
        })?;

        let duration = start.elapsed();
        let status = response.status();
        tracing::debug!(
            module = "http",
            http.method = "POST",
            http.url = %url,
            http.content_type = "multipart/form-data",
            http.status_code = status.as_u16(),
            http.duration_ms = duration.as_millis(),
            "HTTP multipart response received"
        );

        HttpResponse::from_reqwest_response(response, self.config.max_response_body_size)
    }

    // ========================================================================
    // Internal Methods
    // ========================================================================

    fn build_request(
        &self,
        method: HttpMethod,
        url: &str,
        config: RequestConfig,
    ) -> reqwest::blocking::RequestBuilder {
        let mut request = match method {
            HttpMethod::Get => self.client.get(url),
            HttpMethod::Post => self.client.post(url),
            HttpMethod::Put => self.client.put(url),
            HttpMethod::Delete => self.client.delete(url),
            HttpMethod::Patch => self.client.patch(url),
        };

        // 添加 body（如果有）
        if let Some(body) = &config.body {
            request = request.json(body);
        }

        // 应用通用配置（query, auth, headers, timeout）
        request = apply_common_config(
            request,
            &config.query,
            &config.auth,
            &config.headers,
            config.timeout,
        );

        request
    }

    fn build_multipart_request(
        &self,
        url: &str,
        mut config: MultipartRequestConfig,
    ) -> reqwest::blocking::RequestBuilder {
        // 添加请求 ID（用于追踪）
        static REQUEST_ID: AtomicU64 = AtomicU64::new(0);
        let request_id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);

        let mut request = self.client.post(url);

        // 添加 multipart form 数据（必须）
        if let Some(multipart) = config.multipart.take() {
            request = request.multipart(multipart);
        }

        request = request.header("X-Request-ID", request_id.to_string());

        // 应用认证（multipart 需要特殊处理 Bearer token）
        if let Some(auth) = &config.auth {
            request = apply_auth_for_multipart(request, auth);
        }

        // 应用 query 和 headers
        if let Some(query) = &config.query {
            request = request.query(query);
        }

        if let Some(headers) = &config.headers {
            for (key, value) in headers.iter() {
                request = request.header(key, value);
            }
        }

        // 设置超时
        let timeout_duration = config.timeout.unwrap_or_else(|| Duration::from_secs(30));
        request = request.timeout(timeout_duration);

        request
    }

    fn execute_request(
        &self,
        method: HttpMethod,
        url: &str,
        mut config: RequestConfig,
    ) -> Result<HttpResponse> {
        let retry_config = config.retry_config.take();
        let method_str = method.to_string();
        let operation_name = format!("{} {}", method_str, url);
        let max_response_body_size = self.config.max_response_body_size;

        // 在闭包外部克隆配置（不包含 retry_config）
        let mut request_config = config.clone();
        request_config.retry_config = None;

        Self::execute_with_retry(
            || {
                let span = tracing::span!(
                    tracing::Level::DEBUG,
                    "http.client.request",
                    module = "http",
                    http.method = %method_str,
                    http.url = %url,
                );
                let _guard = span.enter();
                let start = Instant::now();

                let response = self
                    .build_request(method, url, request_config.clone())
                    .send()
                    .map_err(|e| {
                        let duration = start.elapsed();
                        let error = Self::handle_request_error(e, url, &method_str);
                        tracing::error!(
                            module = "http",
                            http.method = %method_str,
                            http.url = %url,
                            http.duration_ms = duration.as_millis(),
                            error = %error,
                            "HTTP request failed"
                        );
                        error
                    })?;

                let duration = start.elapsed();
                let status = response.status();
                tracing::debug!(
                    module = "http",
                    http.method = %method_str,
                    http.url = %url,
                    http.status_code = status.as_u16(),
                    http.duration_ms = duration.as_millis(),
                    "HTTP response received"
                );

                HttpResponse::from_reqwest_response(response, max_response_body_size)
            },
            retry_config.as_ref(),
            &operation_name,
        )
    }

    fn execute_with_retry<F, T>(
        operation: F,
        retry_config: Option<&HttpRetryConfig>,
        operation_name: &str,
    ) -> Result<T>
    where
        F: Fn() -> Result<T>,
    {
        if let Some(config) = retry_config {
            let retry_result = HttpRetry::retry(operation, config, operation_name)?;
            Ok(retry_result.result)
        } else {
            operation()
        }
    }

    fn handle_request_error(error: reqwest::Error, url: &str, method: &str) -> ClientHttpError {
        // 检查是否是网络超时
        if error.is_timeout() {
            return ClientHttpError::Timeout {
                url: url.to_owned(),
                method: method.to_owned(),
            };
        }

        // 检查是否是连接失败
        if error.is_connect() {
            return ClientHttpError::ConnectionFailed {
                url: url.to_owned(),
                method: method.to_owned(),
            };
        }

        // 检查是否是速率限制（429）
        if let Some(status) = error.status() {
            if status.as_u16() == 429 {
                return ClientHttpError::RateLimitExceeded {
                    url: url.to_owned(),
                    method: method.to_owned(),
                };
            }
        }

        // 其他错误，使用通用消息
        ClientHttpError::RequestFailed {
            method: method.to_owned(),
            url: url.to_owned(),
            source: error,
        }
    }
}
