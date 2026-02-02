//! HTTP 重试配置

/// 重试结果
#[derive(Debug, Clone)]
pub struct RetryResult<T> {
    /// 操作结果
    pub result: T,
    /// 重试次数
    pub retry_count: u32,
    /// 是否成功（第一次尝试就成功）
    pub succeeded_on_first_attempt: bool,
}

/// HTTP 重试配置
///
/// 用于配置 HTTP 请求的重试策略，支持指数退避算法。
#[derive(Debug, Clone)]
pub struct HttpRetryConfig {
    /// 最大重试次数（默认：3）
    pub max_retries: u32,
    /// 初始延迟（秒，默认：1）
    pub initial_delay: u64,
    /// 最大延迟（秒，默认：30）
    pub max_delay: u64,
    /// 退避倍数（默认：2.0，指数退避）
    pub backoff_multiplier: f64,
}

impl Default for HttpRetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: 1,
            max_delay: 30,
            backoff_multiplier: 2.0,
        }
    }
}

impl HttpRetryConfig {
    /// 创建新的 HttpRetryConfig
    ///
    /// 使用默认值创建重试配置。
    ///
    /// # 返回
    ///
    /// 返回 `HttpRetryConfig` 结构体，使用默认配置：
    /// - `max_retries`: 3
    /// - `initial_delay`: 1 秒
    /// - `max_delay`: 30 秒
    /// - `backoff_multiplier`: 2.0（指数退避）
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_initial_delay(mut self, initial_delay: u64) -> Self {
        self.initial_delay = initial_delay;
        self
    }
}
