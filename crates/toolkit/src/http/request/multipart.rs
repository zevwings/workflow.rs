//! Multipart 请求配置

use std::time::Duration;

use reqwest::blocking::multipart;
use reqwest::header::HeaderMap;

use crate::http::auth::Authorization;
use crate::http::{HttpRetryConfig, IntoHeaderMap};

/// Multipart 请求配置
///
/// 用于 multipart/form-data 请求的配置，支持文件上传等功能。
///
/// # 示例
///
/// ```rust,no_run
/// use toolkit::http::{HttpClient, MultipartRequestConfig};
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
#[derive(Debug, Default)]
pub struct MultipartRequestConfig {
    /// Multipart form 数据
    pub multipart: Option<multipart::Form>,
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

impl MultipartRequestConfig {
    /// 创建新的 MultipartRequestConfig，使用默认值
    ///
    /// # 返回
    ///
    /// 返回一个所有字段都为 `None` 的 `MultipartRequestConfig` 实例。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use toolkit::http::MultipartRequestConfig;
    ///
    /// let config = MultipartRequestConfig::new();
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
    /// use toolkit::http::MultipartRequestConfig;
    /// use reqwest::blocking::multipart;
    ///
    /// let form = multipart::Form::new();
    /// let config = MultipartRequestConfig::new()
    ///     .multipart(form);
    /// ```
    pub fn multipart(mut self, multipart: multipart::Form) -> Self {
        self.multipart = Some(multipart);
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
    pub fn headers(mut self, headers: impl IntoHeaderMap) -> Self {
        self.headers = Some(headers.into_header_map());
        self
    }
}

#[cfg(test)]
mod tests {
    use reqwest::blocking::multipart;
    use reqwest::header::HeaderMap;

    use super::*;

    // ==================== MultipartRequestConfig 基础测试 ====================

    #[test]
    fn test_multipart_request_config_new() {
        let config = MultipartRequestConfig::new();

        assert!(config.multipart.is_none());
        assert!(config.query.is_none());
        assert!(config.auth.is_none());
        assert!(config.headers.is_none());
        assert!(config.timeout.is_none());
        assert!(config.retry_config.is_none());
    }

    #[test]
    fn test_multipart_request_config_default() {
        let config = MultipartRequestConfig::default();

        assert!(config.multipart.is_none());
        assert!(config.query.is_none());
        assert!(config.auth.is_none());
        assert!(config.headers.is_none());
        assert!(config.timeout.is_none());
        assert!(config.retry_config.is_none());
    }

    // ==================== MultipartRequestConfig Builder 测试 ====================

    #[test]
    fn test_multipart_request_config_multipart() {
        let form = multipart::Form::new().text("key", "value");
        let config = MultipartRequestConfig::new().multipart(form);

        assert!(config.multipart.is_some());
    }

    #[test]
    fn test_multipart_request_config_auth() {
        let auth = Authorization::bearer("token");
        let config = MultipartRequestConfig::new().auth(auth);

        assert!(config.auth.is_some());
    }

    #[test]
    fn test_multipart_request_config_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Custom-Header", "value".parse().unwrap());
        let config = MultipartRequestConfig::new().headers(&headers);

        assert!(config.headers.is_some());
    }

    // ==================== MultipartRequestConfig 链式调用测试 ====================

    #[test]
    fn test_multipart_request_config_chaining() {
        let form = multipart::Form::new().text("key", "value");
        let auth = Authorization::bearer("token");
        let mut headers = HeaderMap::new();
        headers.insert("X-Custom-Header", "value".parse().unwrap());

        let config = MultipartRequestConfig::new()
            .multipart(form)
            .auth(auth)
            .headers(&headers);

        assert!(config.multipart.is_some());
        assert!(config.auth.is_some());
        assert!(config.headers.is_some());
    }

    // ==================== MultipartRequestConfig Retry 配置测试 ====================

    #[test]
    fn test_multipart_request_config_retry_ignored() {
        // 注意：MultipartRequestConfig 没有 retry 方法
        // 但可以通过直接设置 retry_config 字段来测试
        // 在实际使用中，post_multipart 会忽略 retry_config 并发出警告
        let mut config = MultipartRequestConfig::new();
        config.retry_config = Some(HttpRetryConfig::new());

        // 验证 retry_config 可以被设置（虽然会被忽略）
        assert!(config.retry_config.is_some());
    }
}
