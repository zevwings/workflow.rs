//! HTTP 客户端配置

use std::time::Duration;

/// HTTP 客户端配置
///
/// 用于配置 HTTP 客户端的连接池、超时、TLS 等选项。
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// 连接池最大连接数（默认：100）
    pub pool_max_idle_per_host: usize,
    /// Keep-alive 超时时间（默认：90 秒）
    pub keep_alive_timeout: Duration,
    /// 连接超时时间（默认：10 秒）
    pub connect_timeout: Duration,
    /// 请求超时时间（默认：30 秒）
    pub timeout: Duration,
    /// User-Agent 字符串（默认：workflow/{version}）
    pub user_agent: String,
    /// 是否验证 TLS 证书（默认：true）
    pub tls_verify: bool,
    /// 最大请求体大小（字节，默认：10MB）
    pub max_request_body_size: usize,
    /// 最大响应体大小（字节，默认：100MB）
    pub max_response_body_size: usize,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            pool_max_idle_per_host: 100,
            keep_alive_timeout: Duration::from_secs(90),
            connect_timeout: Duration::from_secs(10),
            timeout: Duration::from_secs(30),
            user_agent: format!("workflow/{}", env!("CARGO_PKG_VERSION")),
            tls_verify: true,
            max_request_body_size: 10 * 1024 * 1024,   // 10MB
            max_response_body_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

impl HttpClientConfig {
    /// 创建新的 HttpClientConfig，使用默认值
    ///
    /// # 返回
    ///
    /// 返回使用默认配置的 `HttpClientConfig` 实例。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置连接池最大连接数
    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.pool_max_idle_per_host = max;
        self
    }

    /// 设置 Keep-alive 超时时间
    pub fn keep_alive_timeout(mut self, timeout: Duration) -> Self {
        self.keep_alive_timeout = timeout;
        self
    }

    /// 设置连接超时时间
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// 设置请求超时时间
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

    /// 设置最大请求体大小
    pub fn max_request_body_size(mut self, size: usize) -> Self {
        self.max_request_body_size = size;
        self
    }

    /// 设置最大响应体大小
    pub fn max_response_body_size(mut self, size: usize) -> Self {
        self.max_response_body_size = size;
        self
    }
}
