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

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;
    use pretty_assertions::assert_eq;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::time::Duration;

    // ==================== RequestConfig Creation Tests ====================

    /// 测试创建新的 RequestConfig
    ///
    /// ## 测试目的
    /// 验证 RequestConfig::new() 能够创建一个空的配置。
    ///
    /// ## 测试场景
    /// 1. 调用 new() 创建配置
    /// 2. 验证所有字段为 None
    ///
    /// ## 预期结果
    /// - 所有字段（body、query、auth、headers、timeout）都为 None
    #[test]
    fn test_request_config_new_with_no_parameters_creates_empty_config() {
        // Arrange: 准备创建新配置

        // Act: 创建新的 RequestConfig
        let config = RequestConfig::<Value, Value>::new();

        // Assert: 验证所有字段为 None
        assert!(config.body.is_none());
        assert!(config.query.is_none());
        assert!(config.auth.is_none());
        assert!(config.headers.is_none());
        assert!(config.timeout.is_none());
    }

    /// 测试创建默认的 RequestConfig
    ///
    /// ## 测试目的
    /// 验证 RequestConfig::default() 能够创建一个空的配置。
    ///
    /// ## 测试场景
    /// 1. 调用 default() 创建配置
    /// 2. 验证所有字段为 None
    ///
    /// ## 预期结果
    /// - 所有字段都为 None
    #[test]
    fn test_request_config_default_with_no_parameters_creates_empty_config() {
        // Arrange: 准备创建默认配置

        // Act: 创建默认的 RequestConfig
        let config = RequestConfig::<Value, Value>::default();

        // Assert: 验证所有字段为 None
        assert!(config.body.is_none());
        assert!(config.query.is_none());
        assert!(config.auth.is_none());
        assert!(config.headers.is_none());
        assert!(config.timeout.is_none());
    }

    // ==================== RequestConfig Builder Tests ====================

    /// 测试设置 RequestConfig 的请求体
    ///
    /// ## 测试目的
    /// 验证 RequestConfig::body() 能够设置 JSON 请求体。
    ///
    /// ## 测试场景
    /// 1. 创建 JSON 请求体
    /// 2. 使用 body() 方法设置请求体
    /// 3. 验证请求体已设置
    ///
    /// ## 预期结果
    /// - 请求体被正确设置
    #[test]
    fn test_request_config_body_with_json_value_sets_body_return_ok() -> Result<()> {
        // Arrange: 准备 JSON 请求体
        let body = serde_json::json!({"key": "value"});

        // Act: 设置请求体
        let config = RequestConfig::<Value, Value>::new().body(&body);

        // Assert: 验证请求体已设置
        assert!(config.body.is_some());
        if let Some(body) = config.body {
            assert_eq!(body["key"], "value");
        }
        Ok(())
    }

    /// 测试设置 RequestConfig 的查询参数（数组）
    ///
    /// ## 测试目的
    /// 验证 RequestConfig::query() 能够使用数组设置查询参数。
    ///
    /// ## 测试场景
    /// 1. 准备查询参数数组
    /// 2. 使用 query() 方法设置查询参数
    /// 3. 验证查询参数已设置
    ///
    /// ## 预期结果
    /// - 查询参数被正确设置
    #[test]
    fn test_request_config_query_with_array_sets_query() {
        // Arrange: 准备查询参数数组
        let query = [("page", "1"), ("limit", "10")];

        // Act: 设置查询参数
        let config = RequestConfig::<Value, _>::new().query(&query);

        // Assert: 验证查询参数已设置
        assert!(config.query.is_some());
    }

    /// 测试设置 RequestConfig 的查询参数（HashMap）
    ///
    /// ## 测试目的
    /// 验证 RequestConfig::query() 能够使用 HashMap 设置查询参数。
    ///
    /// ## 测试场景
    /// 1. 准备查询参数 HashMap
    /// 2. 使用 query() 方法设置查询参数
    /// 3. 验证查询参数已设置
    ///
    /// ## 预期结果
    /// - 查询参数被正确设置
    #[test]
    fn test_request_config_query_with_hashmap_sets_query() {
        // Arrange: 准备查询参数 HashMap
        let mut params = HashMap::new();
        params.insert("state", "open");
        params.insert("sort", "created");

        // Act: 设置查询参数
        let config = RequestConfig::<Value, _>::new().query(&params);

        // Assert: 验证查询参数已设置
        assert!(config.query.is_some());
    }

    /// 测试设置 RequestConfig 的认证信息
    ///
    /// ## 测试目的
    /// 验证 RequestConfig::auth() 能够设置认证信息。
    ///
    /// ## 测试场景
    /// 1. 创建认证信息
    /// 2. 使用 auth() 方法设置认证信息
    /// 3. 验证认证信息已设置
    ///
    /// ## 预期结果
    /// - 认证信息被正确设置
    #[test]
    fn test_request_config_auth_with_credentials_sets_auth_return_ok() -> Result<()> {
        // Arrange: 准备认证信息
        let auth = Authorization::new("username", "password");

        // Act: 设置认证信息
        let config = RequestConfig::<Value, Value>::new().auth(&auth);

        // Assert: 验证认证信息已设置
        assert!(config.auth.is_some());
        if let Some(config_auth) = config.auth {
            assert_eq!(config_auth.username, "username");
            assert_eq!(config_auth.password, "password");
        }
        Ok(())
    }

    /// 测试设置 RequestConfig 的请求头
    ///
    /// ## 测试目的
    /// 验证 RequestConfig::headers() 能够设置请求头。
    ///
    /// ## 测试场景
    /// 1. 准备请求头映射
    /// 2. 使用 headers() 方法设置请求头
    /// 3. 验证请求头已设置
    ///
    /// ## 预期结果
    /// - 请求头被正确设置
    #[test]
    fn test_request_config_headers_with_header_map_sets_headers_return_ok() -> Result<()> {
        // Arrange: 准备请求头
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("X-Custom-Header", "value".parse()?);

        // Act: 设置请求头
        let config = RequestConfig::<Value, Value>::new().headers(&headers);

        // Assert: 验证请求头已设置
        assert!(config.headers.is_some());
        if let Some(headers) = config.headers {
            assert_eq!(
                headers
                    .get("X-Custom-Header")
                    .ok_or_else(|| color_eyre::eyre::eyre!("Missing header"))?,
                "value"
            );
        }
        Ok(())
    }

    /// 测试设置 RequestConfig 的超时时间
    ///
    /// ## 测试目的
    /// 验证 RequestConfig::timeout() 能够设置超时时间。
    ///
    /// ## 测试场景
    /// 1. 准备超时时间 Duration
    /// 2. 使用 timeout() 方法设置超时时间
    /// 3. 验证超时时间已设置
    ///
    /// ## 预期结果
    /// - 超时时间被正确设置
    #[test]
    fn test_request_config_timeout_with_duration_sets_timeout_return_ok() -> Result<()> {
        // Arrange: 准备超时时间
        let timeout = Duration::from_secs(60);

        // Act: 设置超时时间
        let config = RequestConfig::<Value, Value>::new().timeout(timeout);

        // Assert: 验证超时时间已设置
        assert!(config.timeout.is_some());
        if let Some(t) = config.timeout {
            assert_eq!(t, timeout);
        }
        Ok(())
    }

    /// 测试 RequestConfig 的方法链
    ///
    /// ## 测试目的
    /// 验证 RequestConfig 能够使用方法链设置所有配置选项。
    ///
    /// ## 测试场景
    /// 1. 准备所有配置选项
    /// 2. 使用方法链设置所有选项
    /// 3. 验证所有选项都已设置
    ///
    /// ## 预期结果
    /// - 所有配置选项都被正确设置
    #[test]
    fn test_request_config_chain_with_multiple_options_sets_all_options_return_collect(
    ) -> Result<()> {
        // Arrange: 准备所有配置选项
        let body = serde_json::json!({"data": "test"});
        let query = [("filter", "active")];
        let auth = Authorization::new("user", "pass");
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("X-Request-ID", "12345".parse()?);
        let timeout = Duration::from_secs(30);

        // Act: 链式设置所有配置选项
        let config = RequestConfig::new()
            .body(&body)
            .query(&query)
            .auth(&auth)
            .headers(&headers)
            .timeout(timeout);

        // Assert: 验证所有配置选项已设置
        assert!(config.body.is_some());
        assert!(config.query.is_some());
        assert!(config.auth.is_some());
        assert!(config.headers.is_some());
        assert!(config.timeout.is_some());
        Ok(())
    }

    // ==================== MultipartRequestConfig Creation Tests ====================

    /// 测试创建新的 MultipartRequestConfig
    ///
    /// ## 测试目的
    /// 验证 MultipartRequestConfig::new() 能够创建一个空的配置。
    ///
    /// ## 测试场景
    /// 1. 调用 new() 创建配置
    /// 2. 验证所有字段为 None
    ///
    /// ## 预期结果
    /// - 所有字段（multipart、query、auth、headers、timeout）都为 None
    #[test]
    fn test_multipart_request_config_new_with_no_parameters_creates_empty_config() {
        // Arrange: 准备创建新配置

        // Act: 创建新的 MultipartRequestConfig
        let config = MultipartRequestConfig::<Value>::new();

        // Assert: 验证所有字段为 None
        assert!(config.multipart.is_none());
        assert!(config.query.is_none());
        assert!(config.auth.is_none());
        assert!(config.headers.is_none());
        assert!(config.timeout.is_none());
    }

    /// 测试创建默认的 MultipartRequestConfig
    ///
    /// ## 测试目的
    /// 验证 MultipartRequestConfig::default() 能够创建一个空的配置。
    ///
    /// ## 测试场景
    /// 1. 调用 default() 创建配置
    /// 2. 验证所有字段为 None
    ///
    /// ## 预期结果
    /// - 所有字段都为 None
    #[test]
    fn test_multipart_request_config_default_with_no_parameters_creates_empty_config() {
        // Arrange: 准备创建默认配置

        // Act: 创建默认的 MultipartRequestConfig
        let config = MultipartRequestConfig::<Value>::default();

        // Assert: 验证所有字段为 None
        assert!(config.multipart.is_none());
        assert!(config.query.is_none());
        assert!(config.auth.is_none());
        assert!(config.headers.is_none());
        assert!(config.timeout.is_none());
    }

    // ==================== MultipartRequestConfig Builder Tests ====================

    /// 测试设置 MultipartRequestConfig 的 multipart 表单
    ///
    /// ## 测试目的
    /// 验证 MultipartRequestConfig::multipart() 能够设置 multipart 表单。
    ///
    /// ## 测试场景
    /// 1. 准备 multipart 表单
    /// 2. 使用 multipart() 方法设置表单
    /// 3. 验证表单已设置
    ///
    /// ## 预期结果
    /// - multipart 表单被正确设置
    #[test]
    fn test_multipart_request_config_multipart_with_form_sets_multipart() {
        // Arrange: 准备 multipart 表单
        let form = reqwest::blocking::multipart::Form::new()
            .text("key", "value")
            .text("name", "test");

        // Act: 设置 multipart 表单
        let config = MultipartRequestConfig::<Value>::new().multipart(form);

        // Assert: 验证 multipart 表单已设置
        assert!(config.multipart.is_some());
    }

    /// 测试设置 MultipartRequestConfig 的查询参数
    ///
    /// ## 测试目的
    /// 验证 MultipartRequestConfig::query() 能够设置查询参数。
    ///
    /// ## 测试场景
    /// 1. 准备查询参数 JSON
    /// 2. 使用 query() 方法设置查询参数
    /// 3. 验证查询参数已设置
    ///
    /// ## 预期结果
    /// - 查询参数被正确设置
    #[test]
    fn test_multipart_request_config_query_with_json_sets_query() {
        // Arrange: 准备查询参数 JSON
        let query = serde_json::json!({"param": "value"});

        // Act: 设置查询参数
        let config = MultipartRequestConfig::new().query(query);

        // Assert: 验证查询参数已设置
        assert!(config.query.is_some());
    }

    /// 测试设置 MultipartRequestConfig 的认证信息
    ///
    /// ## 测试目的
    /// 验证 MultipartRequestConfig::auth() 能够设置认证信息。
    ///
    /// ## 测试场景
    /// 1. 创建认证信息
    /// 2. 使用 auth() 方法设置认证信息
    /// 3. 验证认证信息已设置
    ///
    /// ## 预期结果
    /// - 认证信息被正确设置
    #[test]
    fn test_multipart_request_config_auth_with_credentials_sets_auth_return_ok() -> Result<()> {
        // Arrange: 准备认证信息
        let auth = Authorization::new("username", "password");
        let auth_clone = auth.clone();

        // Act: 设置认证信息
        let config = MultipartRequestConfig::<Value>::new().auth(auth);

        // Assert: 验证认证信息已设置
        assert!(config.auth.is_some());
        if let Some(config_auth) = config.auth {
            assert_eq!(config_auth.username, auth_clone.username);
            assert_eq!(config_auth.password, auth_clone.password);
        }
        Ok(())
    }

    /// 测试设置 MultipartRequestConfig 的请求头
    ///
    /// ## 测试目的
    /// 验证 MultipartRequestConfig::headers() 能够设置请求头。
    ///
    /// ## 测试场景
    /// 1. 准备请求头映射
    /// 2. 使用 headers() 方法设置请求头
    /// 3. 验证请求头已设置
    ///
    /// ## 预期结果
    /// - 请求头被正确设置
    #[test]
    fn test_multipart_request_config_headers_with_header_map_sets_headers_return_ok() -> Result<()>
    {
        // Arrange: 准备请求头
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("X-Custom-Header", "value".parse()?);

        // Act: 设置请求头
        let config = MultipartRequestConfig::<Value>::new().headers(headers.clone());

        // Assert: 验证请求头已设置
        assert!(config.headers.is_some());
        if let Some(headers) = config.headers {
            assert_eq!(
                headers
                    .get("X-Custom-Header")
                    .ok_or_else(|| color_eyre::eyre::eyre!("Missing header"))?,
                "value"
            );
        }
        Ok(())
    }

    /// 测试设置 MultipartRequestConfig 的超时时间
    ///
    /// ## 测试目的
    /// 验证 MultipartRequestConfig::timeout() 能够设置超时时间。
    ///
    /// ## 测试场景
    /// 1. 准备超时时间 Duration
    /// 2. 使用 timeout() 方法设置超时时间
    /// 3. 验证超时时间已设置
    ///
    /// ## 预期结果
    /// - 超时时间被正确设置
    #[test]
    fn test_multipart_request_config_timeout_with_duration_sets_timeout_return_ok() -> Result<()> {
        // Arrange: 准备超时时间
        let timeout = Duration::from_secs(120);

        // Act: 设置超时时间
        let config = MultipartRequestConfig::<Value>::new().timeout(timeout);

        // Assert: 验证超时时间已设置
        assert!(config.timeout.is_some());
        if let Some(t) = config.timeout {
            assert_eq!(t, timeout);
        }
        Ok(())
    }

    /// 测试 MultipartRequestConfig 的方法链
    ///
    /// ## 测试目的
    /// 验证 MultipartRequestConfig 能够使用方法链设置所有配置选项。
    ///
    /// ## 测试场景
    /// 1. 准备所有配置选项
    /// 2. 使用方法链设置所有选项
    /// 3. 验证所有选项都已设置
    ///
    /// ## 预期结果
    /// - 所有配置选项都被正确设置
    #[test]
    fn test_multipart_request_config_chain_with_multiple_options_sets_all_options_return_collect(
    ) -> Result<()> {
        // Arrange: 准备所有配置选项
        let form = reqwest::blocking::multipart::Form::new().text("file", "content");
        let query = serde_json::json!({"param": "value"});
        let auth = Authorization::new("user", "pass");
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("X-Upload-ID", "67890".parse()?);
        let timeout = Duration::from_secs(60);

        // Act: 链式设置所有配置选项
        let config = MultipartRequestConfig::new()
            .multipart(form)
            .query(query)
            .auth(auth)
            .headers(headers)
            .timeout(timeout);

        // Assert: 验证所有配置选项已设置
        assert!(config.multipart.is_some());
        assert!(config.query.is_some());
        assert!(config.auth.is_some());
        assert!(config.headers.is_some());
        assert!(config.timeout.is_some());
        Ok(())
    }
}
