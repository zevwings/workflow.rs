//! HTTP 客户端配置

use std::time::Duration;

/// HTTP 客户端配置
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// 基础 URL（可选）
    ///
    /// 设置后，请求方法（如 `get`、`post`）可以只传入路径（如 `/data`），
    /// 会自动与 base_url 拼接成完整 URL。
    pub base_url: Option<String>,
    /// 连接超时时间
    pub connect_timeout: Duration,
    /// 请求超时时间
    pub timeout: Duration,
    /// User-Agent
    pub user_agent: String,
    /// 是否验证 TLS 证书
    pub tls_verify: bool,
    /// 最大响应体大小
    pub max_response_body_size: usize,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            connect_timeout: Duration::from_secs(10),
            timeout: Duration::from_secs(30),
            user_agent: format!("workflow/{}", env!("CARGO_PKG_VERSION")),
            tls_verify: true,
            max_response_body_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

impl HttpClientConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置基础 URL
    ///
    /// 设置后，请求方法可以只传入路径，会自动与 base_url 拼接。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let config = HttpClientConfig::new()
    ///     .base_url("https://api.example.com");
    ///
    /// let client = HttpClient::with_config(config)?;
    ///
    /// // 只需传入路径，会自动拼接为 https://api.example.com/users
    /// let response = client.get("/users").send()?;
    /// ```
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        // 移除尾部斜杠，避免拼接时出现双斜杠
        self.base_url = Some(url.trim_end_matches('/').to_string());
        self
    }

    /// 设置连接超时
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// 设置请求超时
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 设置 User-Agent
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// 设置是否验证 TLS 证书
    pub fn tls_verify(mut self, verify: bool) -> Self {
        self.tls_verify = verify;
        self
    }

    /// 设置最大响应体大小
    pub fn max_response_body_size(mut self, size: usize) -> Self {
        self.max_response_body_size = size;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = HttpClientConfig::default();
        assert!(config.base_url.is_none());
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.tls_verify);
        assert_eq!(config.max_response_body_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_config_builder() {
        let config = HttpClientConfig::new()
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(5))
            .user_agent("test-agent")
            .tls_verify(false);

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.connect_timeout, Duration::from_secs(5));
        assert_eq!(config.user_agent, "test-agent");
        assert!(!config.tls_verify);
    }

    #[test]
    fn test_config_base_url() {
        let config = HttpClientConfig::new().base_url("https://api.example.com");

        assert_eq!(config.base_url, Some("https://api.example.com".to_string()));
    }

    #[test]
    fn test_config_base_url_trailing_slash() {
        // 尾部斜杠应被移除
        let config = HttpClientConfig::new().base_url("https://api.example.com/");

        assert_eq!(config.base_url, Some("https://api.example.com".to_string()));
    }

    #[test]
    fn test_config_base_url_with_path() {
        let config = HttpClientConfig::new().base_url("https://api.example.com/v1/");

        assert_eq!(
            config.base_url,
            Some("https://api.example.com/v1".to_string())
        );
    }
}
