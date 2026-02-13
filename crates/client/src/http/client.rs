//! HTTP 客户端 trait（定义层）
//!
//! 定义 HTTP 客户端的核心能力，实现层（如 infra）提供具体实现。
//! 链式 API（get/post、.body()、.auth()、.multipart()、.send()）在定义层提供，
//! 外部通过 `HttpClientHolder` 使用，不必关心 infra 如何发起请求。

use std::{collections::HashMap, sync::Arc, time::Duration};

use super::{HttpError, HttpMethod, HttpRequest, HttpResponse};
use super::{MultipartRequest, RequestBuilder};

/// HTTP 客户端 trait
///
/// 定义 `execute` 和 `execute_multipart` 核心能力。
/// 实现层（如 infra）提供具体实现。
pub trait HttpClient: Send + Sync + 'static {
    /// 执行 HTTP 请求（JSON body）
    fn execute(&self, req: HttpRequest) -> Result<HttpResponse, HttpError>;

    /// 执行 Multipart 请求
    ///
    /// 实现层将 `MultipartRequest` 转换为具体 HTTP 库的 multipart 格式并发送。
    fn execute_multipart(
        &self,
        method: HttpMethod,
        url: &str,
        multipart: MultipartRequest,
        query: Option<serde_json::Value>,
        headers: HashMap<String, String>,
        auth: Option<super::Authorization>,
        timeout: Option<Duration>,
    ) -> Result<HttpResponse, HttpError>;

    /// 基础 URL（可选）
    ///
    /// 设置后，相对路径（以 `/` 开头）会自动拼接。默认返回 `None`。
    fn base_url(&self) -> Option<&str> {
        None
    }

    /// 解析 URL：相对路径与 base_url 拼接
    fn resolve_url(&self, url: &str) -> String {
        if url.starts_with('/') {
            if let Some(base) = self.base_url() {
                return format!("{}{}", base, url);
            }
        }
        url.to_string()
    }
}

/// HTTP 客户端持有者
///
/// 包装 `Arc<dyn HttpClient>`，提供 get/post/multipart 链式 API。
/// DI 注入时使用此类型，外部通过 `client::http::HttpClientHolder` 访问。
///
/// # 示例
///
/// ```rust,ignore
/// use client::http::{HttpClientHolder, MultipartRequest};
/// use std::sync::Arc;
///
/// let client: Arc<dyn HttpClient> = Arc::new(ReqwestHttpClient::new()?);
/// let holder = HttpClientHolder::new(client);
///
/// // GET
/// let response = holder.get("/users").send()?;
///
/// // POST with JSON
/// let response = holder.post("/users").body(&payload)?.send()?;
///
/// // Multipart
/// let mp = MultipartRequest::new().text("name", "value").file("file", path);
/// let response = holder.post("/upload").multipart(mp).send()?;
/// ```
pub struct HttpClientHolder {
    inner: Arc<dyn HttpClient>,
}

impl HttpClientHolder {
    /// 从 `Arc<dyn HttpClient>` 创建
    pub fn new(inner: Arc<dyn HttpClient>) -> Self {
        Self { inner }
    }

    /// GET 请求
    pub fn get(&self, url: &str) -> RequestBuilder<'_> {
        RequestBuilder::new(
            self.inner.as_ref(),
            HttpMethod::GET,
            self.inner.resolve_url(url),
        )
    }

    /// POST 请求
    pub fn post(&self, url: &str) -> RequestBuilder<'_> {
        RequestBuilder::new(
            self.inner.as_ref(),
            HttpMethod::POST,
            self.inner.resolve_url(url),
        )
    }

    /// PUT 请求
    pub fn put(&self, url: &str) -> RequestBuilder<'_> {
        RequestBuilder::new(
            self.inner.as_ref(),
            HttpMethod::PUT,
            self.inner.resolve_url(url),
        )
    }

    /// DELETE 请求
    pub fn delete(&self, url: &str) -> RequestBuilder<'_> {
        RequestBuilder::new(
            self.inner.as_ref(),
            HttpMethod::DELETE,
            self.inner.resolve_url(url),
        )
    }

    /// PATCH 请求
    pub fn patch(&self, url: &str) -> RequestBuilder<'_> {
        RequestBuilder::new(
            self.inner.as_ref(),
            HttpMethod::PATCH,
            self.inner.resolve_url(url),
        )
    }

    /// 直接执行 HTTP 请求
    ///
    /// 提供给需要直接构建 `HttpRequest` 的场景使用，避免重复的 match 代码。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let req = HttpRequest {
    ///     method: HttpMethod::POST,
    ///     url: "https://api.example.com/users".to_string(),
    ///     headers: headers,
    ///     body: Some(json_body),
    ///     query: None,
    ///     auth: None,
    ///     timeout: None,
    /// };
    /// let response = holder.execute(req)?;
    /// ```
    pub fn execute(&self, req: HttpRequest) -> Result<HttpResponse, HttpError> {
        self.inner.execute(req)
    }
}

/// HTTP 客户端扩展：为具体类型提供 get/post 等便捷方法
///
/// 当直接使用 `ReqwestHttpClient` 等具体类型时，可调用 `client.get("/foo")`。
pub trait HttpClientExt: HttpClient + Sized {
    /// GET 请求
    fn get(&self, url: &str) -> RequestBuilder<'_> {
        RequestBuilder::new(self, HttpMethod::GET, self.resolve_url(url))
    }

    /// POST 请求
    fn post(&self, url: &str) -> RequestBuilder<'_> {
        RequestBuilder::new(self, HttpMethod::POST, self.resolve_url(url))
    }

    /// PUT 请求
    fn put(&self, url: &str) -> RequestBuilder<'_> {
        RequestBuilder::new(self, HttpMethod::PUT, self.resolve_url(url))
    }

    /// DELETE 请求
    fn delete(&self, url: &str) -> RequestBuilder<'_> {
        RequestBuilder::new(self, HttpMethod::DELETE, self.resolve_url(url))
    }

    /// PATCH 请求
    fn patch(&self, url: &str) -> RequestBuilder<'_> {
        RequestBuilder::new(self, HttpMethod::PATCH, self.resolve_url(url))
    }
}

impl<T: HttpClient + Sized> HttpClientExt for T {}
