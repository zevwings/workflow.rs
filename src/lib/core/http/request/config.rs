//! HTTP 请求配置

use reqwest::header::HeaderMap;
use serde::Serialize;
use std::time::Duration;

use crate::core::http::auth::Authorization;
use crate::core::http::request::headers::IntoHeaderMap;
use crate::core::http::retry::config::HttpRetryConfig;

/// HTTP 请求配置
///
/// 使用 builder 模式构建 HTTP 请求配置。所有字段都是可选的，使用默认值。
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
#[derive(Debug, Clone, Default)]
pub struct RequestConfig {
    /// 可选的请求体（序列化为 JSON）
    pub body: Option<serde_json::Value>,
    /// 可选的查询参数（序列化为 URL 查询字符串）
    pub query: Option<serde_json::Value>,
    /// 可选的认证信息
    pub auth: Option<Authorization>,
    /// 可选的自定义 HTTP Headers
    pub headers: Option<HeaderMap>,
    /// 可选的请求超时时间（如果为 None，使用默认 30 秒）
    pub timeout: Option<Duration>,
    /// 可选的重试配置（如果为 None，不进行重试）
    pub retry_config: Option<HttpRetryConfig>,
}

impl RequestConfig {
    /// 创建新的 RequestConfig，使用默认值
    ///
    /// # 返回
    ///
    /// 返回一个所有字段都为 `None` 的 `RequestConfig` 实例。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::RequestConfig;
    ///
    /// let config = RequestConfig::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置请求体
    ///
    /// # 参数
    ///
    /// * `body` - 请求体，必须实现 `Serialize` trait
    ///
    /// # 返回
    ///
    /// 返回 `Self`，支持链式调用。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::RequestConfig;
    ///
    /// let body = serde_json::json!({"key": "value"});
    /// let config = RequestConfig::new().body(&body);
    /// ```
    pub fn body<T: Serialize>(mut self, body: &T) -> Self {
        self.body = serde_json::to_value(body)
            .map_err(|e| {
                tracing::warn!("Failed to serialize request body: {}", e);
            })
            .ok();
        self
    }

    /// 设置查询参数
    ///
    /// # 参数
    ///
    /// * `query` - 查询参数，必须实现 `Serialize` trait
    ///
    /// # 返回
    ///
    /// 返回 `Self`，支持链式调用。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::RequestConfig;
    ///
    /// let query = serde_json::json!({"page": "1"});
    /// let config = RequestConfig::new().query(&query);
    /// ```
    pub fn query<T: Serialize>(mut self, query: &T) -> Self {
        self.query = serde_json::to_value(query)
            .map_err(|e| {
                tracing::warn!("Failed to serialize query parameters: {}", e);
            })
            .ok();
        self
    }

    /// 设置认证信息
    ///
    /// # 参数
    ///
    /// * `auth` - 认证信息
    ///
    /// # 返回
    ///
    /// 返回 `Self`，支持链式调用。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::{RequestConfig, Authorization};
    ///
    /// let auth = Authorization::bearer("api_token");
    /// let config = RequestConfig::new().auth(auth);
    /// ```
    pub fn auth(mut self, auth: Authorization) -> Self {
        self.auth = Some(auth);
        self
    }

    /// 设置 HTTP Headers
    ///
    /// # 参数
    ///
    /// * `headers` - HTTP Headers（可以是引用或拥有值）
    ///
    /// # 返回
    ///
    /// 返回 `Self`，支持链式调用。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::RequestConfig;
    /// use reqwest::header::HeaderMap;
    ///
    /// let mut headers = HeaderMap::new();
    /// headers.insert("X-Custom-Header", "value".parse().unwrap());
    /// let config = RequestConfig::new().headers(&headers);
    /// ```
    pub fn headers(mut self, headers: impl IntoHeaderMap) -> Self {
        self.headers = Some(headers.into_header_map());
        self
    }

    /// 设置超时时间
    ///
    /// # 参数
    ///
    /// * `timeout` - 请求超时时间
    ///
    /// # 返回
    ///
    /// 返回 `Self`，支持链式调用。
    ///
    /// # 注意
    ///
    /// 如果不设置超时时间，将使用默认的 30 秒超时。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::RequestConfig;
    /// use std::time::Duration;
    ///
    /// let config = RequestConfig::new()
    ///     .timeout(Duration::from_secs(60));
    /// ```
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// 设置重试配置
    ///
    /// # 参数
    ///
    /// * `retry_config` - 重试配置
    ///
    /// # 返回
    ///
    /// 返回 `Self`，支持链式调用。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::{RequestConfig, HttpRetryConfig};
    ///
    /// let config = RequestConfig::new()
    ///     .retry(HttpRetryConfig::new());
    /// ```
    pub fn retry(mut self, retry_config: HttpRetryConfig) -> Self {
        self.retry_config = Some(retry_config);
        self
    }
}
