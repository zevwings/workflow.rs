//! HTTP 请求配置

use reqwest::blocking::multipart;
use reqwest::header::HeaderMap;
use serde::Serialize;
use std::time::Duration;

use super::auth::Authorization;

/// HTTP 请求配置
///
/// # 示例
///
/// ```rust,no_run
/// use serde_json::Value;
/// use workflow::base::http::{HttpClient, RequestConfig};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = HttpClient::global()?;
/// let query = [("page", "1")];
/// let config = RequestConfig::<Value, _>::new()
///     .query(&query);
/// let response = client.get("https://api.example.com", config)?;
/// # Ok(())
/// # }
/// ```
pub struct RequestConfig<'a, B, Q: ?Sized> {
    /// 可选的请求体（实现 `Serialize` trait）
    pub body: Option<&'a B>,
    /// 可选的查询参数（实现 `Serialize` trait）
    pub query: Option<&'a Q>,
    /// 可选的 Basic Authentication 认证信息
    pub auth: Option<&'a Authorization>,
    /// 可选的自定义 HTTP Headers
    pub headers: Option<&'a HeaderMap>,
    /// 可选的请求超时时间（如果为 None，使用默认 30 秒）
    pub timeout: Option<Duration>,
}

impl<'a, B, Q: ?Sized> Default for RequestConfig<'a, B, Q> {
    fn default() -> Self {
        Self {
            body: None,
            query: None,
            auth: None,
            headers: None,
            timeout: None,
        }
    }
}

impl<'a, B, Q: ?Sized> RequestConfig<'a, B, Q> {
    /// 创建新的 RequestConfig，使用默认值
    ///
    /// # 返回
    ///
    /// 返回一个所有字段都为 `None` 的 `RequestConfig` 实例。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::base::http::RequestConfig;
    ///
    /// let config = RequestConfig::<serde_json::Value, serde_json::Value>::new();
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
    /// use serde_json::Value;
    /// use workflow::base::http::RequestConfig;
    ///
    /// let body = serde_json::json!({"key": "value"});
    /// let config = RequestConfig::<Value, Value>::new().body(&body);
    /// ```
    pub fn body(mut self, body: &'a B) -> Self {
        self.body = Some(body);
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
    /// use serde_json::Value;
    /// use workflow::base::http::RequestConfig;
    ///
    /// // 使用元组数组
    /// let query = [("page", "1"), ("per_page", "10")];
    /// let config = RequestConfig::<Value, _>::new().query(&query);
    ///
    /// // 使用 HashMap
    /// use std::collections::HashMap;
    /// let mut params = HashMap::new();
    /// params.insert("state", "open");
    /// let config = RequestConfig::<Value, _>::new().query(&params);
    /// ```
    pub fn query(mut self, query: &'a Q) -> Self {
        self.query = Some(query);
        self
    }

    /// 设置认证信息
    ///
    /// # 参数
    ///
    /// * `auth` - Basic Authentication 认证信息
    ///
    /// # 返回
    ///
    /// 返回 `Self`，支持链式调用。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use serde_json::Value;
    /// use workflow::base::http::{RequestConfig, Authorization};
    ///
    /// let auth = Authorization::new("user@example.com", "api_token");
    /// let config = RequestConfig::<Value, Value>::new().auth(&auth);
    /// ```
    pub fn auth(mut self, auth: &'a Authorization) -> Self {
        self.auth = Some(auth);
        self
    }

    /// 设置 HTTP Headers
    ///
    /// # 参数
    ///
    /// * `headers` - HTTP Headers
    ///
    /// # 返回
    ///
    /// 返回 `Self`，支持链式调用。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use serde_json::Value;
    /// use workflow::base::http::RequestConfig;
    /// use reqwest::header::HeaderMap;
    ///
    /// let mut headers = HeaderMap::new();
    /// headers.insert("X-Custom-Header", "value".parse().unwrap());
    /// let config = RequestConfig::<Value, Value>::new().headers(&headers);
    /// ```
    pub fn headers(mut self, headers: &'a HeaderMap) -> Self {
        self.headers = Some(headers);
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
    /// use serde_json::Value;
    /// use workflow::base::http::RequestConfig;
    /// use std::time::Duration;
    ///
    /// let config = RequestConfig::<Value, Value>::new()
    ///     .timeout(Duration::from_secs(60));
    /// ```
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Multipart 请求配置
///
/// 用于 multipart/form-data 请求的配置，支持文件上传等功能。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::http::{HttpClient, MultipartRequestConfig};
/// use reqwest::blocking::multipart;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = HttpClient::global()?;
/// let form = multipart::Form::new();
/// let config = MultipartRequestConfig::<()>::new()
///     .multipart(form);
/// let response = client.post_multipart("https://api.example.com/upload", config)?;
/// # Ok(())
/// # }
/// ```
pub struct MultipartRequestConfig<Q> {
    /// Multipart form 数据
    pub multipart: Option<multipart::Form>,
    /// 可选的查询参数（实现 `Serialize` trait）
    pub query: Option<Q>,
    /// 可选的 Basic Authentication 认证信息
    pub auth: Option<Authorization>,
    /// 可选的自定义 HTTP Headers
    pub headers: Option<HeaderMap>,
    /// 可选的请求超时时间（如果为 None，使用默认 30 秒）
    pub timeout: Option<Duration>,
}

impl<Q> Default for MultipartRequestConfig<Q> {
    fn default() -> Self {
        Self {
            multipart: None,
            query: None,
            auth: None,
            headers: None,
            timeout: None,
        }
    }
}

impl<Q> MultipartRequestConfig<Q> {
    /// 创建新的 MultipartRequestConfig，使用默认值
    ///
    /// # 返回
    ///
    /// 返回一个所有字段都为 `None` 的 `MultipartRequestConfig` 实例。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::base::http::MultipartRequestConfig;
    ///
    /// let config = MultipartRequestConfig::<serde_json::Value>::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置 multipart form 数据
    ///
    /// # 参数
    ///
    /// * `multipart` - Multipart form 数据
    ///
    /// # 返回
    ///
    /// 返回 `Self`，支持链式调用。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::base::http::MultipartRequestConfig;
    /// use reqwest::blocking::multipart;
    ///
    /// let form = multipart::Form::new();
    /// let config = MultipartRequestConfig::<serde_json::Value>::new()
    ///     .multipart(form);
    /// ```
    pub fn multipart(mut self, multipart: multipart::Form) -> Self {
        self.multipart = Some(multipart);
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
    pub fn query(mut self, query: Q) -> Self
    where
        Q: Serialize,
    {
        self.query = Some(query);
        self
    }

    /// 设置认证信息
    ///
    /// # 参数
    ///
    /// * `auth` - Basic Authentication 认证信息
    ///
    /// # 返回
    ///
    /// 返回 `Self`，支持链式调用。
    pub fn auth(mut self, auth: Authorization) -> Self {
        self.auth = Some(auth);
        self
    }

    /// 设置 HTTP Headers
    ///
    /// # 参数
    ///
    /// * `headers` - HTTP Headers
    ///
    /// # 返回
    ///
    /// 返回 `Self`，支持链式调用。
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers = Some(headers);
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
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}
