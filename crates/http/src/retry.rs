//! HTTP 重试配置和逻辑

use std::thread;
use std::time::{Duration, Instant};

use rand::Rng;

use super::error::HttpError;

/// 重试结果
///
/// 包含操作结果和重试统计信息。
#[derive(Debug)]
pub struct RetryResult<T> {
    /// 操作结果
    pub result: T,
    /// 重试次数（不包括首次尝试）
    pub retry_count: u32,
    /// 是否首次尝试成功
    pub succeeded_on_first_attempt: bool,
    /// 总耗时
    pub total_duration: Duration,
}

impl<T> RetryResult<T> {
    /// 获取操作结果
    pub fn into_result(self) -> T {
        self.result
    }
}

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 初始重试延迟
    pub initial_delay: Duration,
    /// 最大重试延迟
    pub max_delay: Duration,
    /// 延迟倍数（指数退避）
    pub backoff_factor: f64,
    /// 可重试的状态码
    pub retryable_status_codes: Vec<u16>,
    /// 是否启用 jitter（避免雷群效应）
    pub jitter: bool,
    /// Jitter 因子（0.0 - 1.0），表示延迟的随机波动范围
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
            retryable_status_codes: vec![429, 500, 502, 503, 504],
            jitter: true,
            jitter_factor: 0.5,
        }
    }
}

impl RetryConfig {
    /// 创建新的重试配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最大重试次数
    pub fn max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    /// 设置初始重试延迟
    pub fn initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// 设置最大重试延迟
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// 设置延迟倍数
    pub fn backoff_factor(mut self, factor: f64) -> Self {
        self.backoff_factor = factor;
        self
    }

    /// 设置可重试的状态码
    pub fn retryable_status_codes(mut self, codes: Vec<u16>) -> Self {
        self.retryable_status_codes = codes;
        self
    }

    /// 添加可重试的状态码
    pub fn add_retryable_status_code(mut self, code: u16) -> Self {
        if !self.retryable_status_codes.contains(&code) {
            self.retryable_status_codes.push(code);
        }
        self
    }

    /// 启用/禁用 jitter
    pub fn jitter(mut self, enabled: bool) -> Self {
        self.jitter = enabled;
        self
    }

    /// 设置 jitter 因子（0.0 - 1.0）
    ///
    /// Jitter 因子表示延迟的随机波动范围。
    /// 例如，因子为 0.5 时，延迟将在 [delay * 0.5, delay * 1.5] 范围内随机选择。
    pub fn jitter_factor(mut self, factor: f64) -> Self {
        self.jitter_factor = factor.clamp(0.0, 1.0);
        self
    }

    /// 计算指定重试次数的延迟（不含 jitter）
    fn base_delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return self.initial_delay;
        }

        let delay_ms =
            self.initial_delay.as_millis() as f64 * self.backoff_factor.powi(attempt as i32);
        let delay = Duration::from_millis(delay_ms as u64);

        if delay > self.max_delay {
            self.max_delay
        } else {
            delay
        }
    }

    /// 计算指定重试次数的延迟（含 jitter）
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_delay = self.base_delay_for_attempt(attempt);

        if !self.jitter || self.jitter_factor == 0.0 {
            return base_delay;
        }

        // 应用 jitter：delay * (1 - jitter_factor + random * 2 * jitter_factor)
        // 例如 jitter_factor = 0.5 时，范围是 [0.5 * delay, 1.5 * delay]
        let mut rng = rand::rng();
        let jitter_range = self.jitter_factor * 2.0;
        let jitter_multiplier = 1.0 - self.jitter_factor + rng.random::<f64>() * jitter_range;

        let jittered_ms = base_delay.as_millis() as f64 * jitter_multiplier;
        Duration::from_millis(jittered_ms.max(1.0) as u64)
    }

    /// 检查错误是否可重试
    pub fn is_retryable(&self, error: &HttpError) -> bool {
        match error {
            HttpError::Timeout { .. } => true,
            HttpError::Connection { .. } => true,
            HttpError::Status { status, .. } => self.retryable_status_codes.contains(status),
            HttpError::Request { source, .. } => source.is_timeout() || source.is_connect(),
            _ => false,
        }
    }
}

/// 重试执行器
pub(crate) struct RetryExecutor<'a> {
    config: &'a RetryConfig,
}

impl<'a> RetryExecutor<'a> {
    /// 创建重试执行器
    pub fn new(config: &'a RetryConfig) -> Self {
        Self { config }
    }

    /// 执行带重试的操作，返回结果和重试统计信息
    pub fn execute_with_result<F, T>(&self, mut operation: F) -> Result<RetryResult<T>, HttpError>
    where
        F: FnMut() -> Result<T, HttpError>,
    {
        let start = Instant::now();
        let mut attempt = 0;

        loop {
            match operation() {
                Ok(result) => {
                    let total_duration = start.elapsed();
                    let retry_count = attempt;

                    if retry_count > 0 {
                        tracing::debug!(
                            retry_count = retry_count,
                            total_duration_ms = total_duration.as_millis(),
                            "Request succeeded after retry"
                        );
                    }

                    return Ok(RetryResult {
                        result,
                        retry_count,
                        succeeded_on_first_attempt: retry_count == 0,
                        total_duration,
                    });
                }
                Err(error) => {
                    if attempt >= self.config.max_retries || !self.config.is_retryable(&error) {
                        if attempt > 0 {
                            return Err(HttpError::RetryExhausted {
                                attempts: attempt + 1,
                                last_error: Box::new(error),
                            });
                        }
                        return Err(error);
                    }

                    let delay = self.config.delay_for_attempt(attempt);
                    tracing::debug!(
                        attempt = attempt + 1,
                        max_retries = self.config.max_retries,
                        delay_ms = delay.as_millis(),
                        jitter = self.config.jitter,
                        error = %error,
                        "Retrying request"
                    );

                    thread::sleep(delay);
                    attempt += 1;
                }
            }
        }
    }

    /// 执行带重试的操作（简化版，仅返回结果）
    pub fn execute<F, T>(&self, operation: F) -> Result<T, HttpError>
    where
        F: FnMut() -> Result<T, HttpError>,
    {
        self.execute_with_result(operation).map(|r| r.result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorContext;
    use crate::HttpMethod;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.backoff_factor, 2.0);
        assert!(config.retryable_status_codes.contains(&429));
        assert!(config.retryable_status_codes.contains(&500));
        assert!(config.jitter);
        assert_eq!(config.jitter_factor, 0.5);
    }

    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfig::new()
            .max_retries(5)
            .initial_delay(Duration::from_millis(200))
            .backoff_factor(1.5)
            .jitter(false)
            .jitter_factor(0.3);

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay, Duration::from_millis(200));
        assert_eq!(config.backoff_factor, 1.5);
        assert!(!config.jitter);
        assert_eq!(config.jitter_factor, 0.3);
    }

    #[test]
    fn test_jitter_factor_clamping() {
        let config = RetryConfig::new().jitter_factor(2.0);
        assert_eq!(config.jitter_factor, 1.0);

        let config = RetryConfig::new().jitter_factor(-0.5);
        assert_eq!(config.jitter_factor, 0.0);
    }

    #[test]
    fn test_base_delay_for_attempt() {
        let config = RetryConfig::new()
            .initial_delay(Duration::from_millis(100))
            .backoff_factor(2.0)
            .max_delay(Duration::from_secs(10))
            .jitter(false);

        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
        assert_eq!(config.delay_for_attempt(3), Duration::from_millis(800));
    }

    #[test]
    fn test_delay_with_jitter() {
        let config = RetryConfig::new()
            .initial_delay(Duration::from_millis(100))
            .backoff_factor(2.0)
            .jitter(true)
            .jitter_factor(0.5);

        // 运行多次，确保 jitter 产生不同的值
        let delays: Vec<Duration> = (0..10).map(|_| config.delay_for_attempt(1)).collect();

        // 基础延迟是 200ms，jitter_factor 0.5 表示范围是 [100ms, 300ms]
        for delay in &delays {
            assert!(delay.as_millis() >= 100);
            assert!(delay.as_millis() <= 300);
        }

        // 检查是否有变化（不是所有值都相同）
        let first = delays[0];
        let has_variation = delays.iter().any(|d| *d != first);
        assert!(has_variation, "Jitter should produce varying delays");
    }

    #[test]
    fn test_delay_capped_at_max() {
        let config = RetryConfig::new()
            .initial_delay(Duration::from_secs(1))
            .backoff_factor(10.0)
            .max_delay(Duration::from_secs(5))
            .jitter(false);

        // 1s * 10^2 = 100s, but capped at 5s
        assert_eq!(config.delay_for_attempt(2), Duration::from_secs(5));
    }

    #[test]
    fn test_is_retryable_timeout() {
        let config = RetryConfig::default();
        let error = HttpError::Timeout {
            context: ErrorContext::boxed("https://example.com", HttpMethod::GET),
        };
        assert!(config.is_retryable(&error));
    }

    #[test]
    fn test_is_retryable_connection() {
        let config = RetryConfig::default();
        let error = HttpError::Connection {
            context: ErrorContext::boxed("https://example.com", HttpMethod::GET),
        };
        assert!(config.is_retryable(&error));
    }

    #[test]
    fn test_is_retryable_status() {
        let config = RetryConfig::default();

        // 429 is retryable
        let error = HttpError::Status {
            status: 429,
            context: ErrorContext::boxed("https://example.com", HttpMethod::GET),
        };
        assert!(config.is_retryable(&error));

        // 404 is not retryable
        let error = HttpError::Status {
            status: 404,
            context: ErrorContext::boxed("https://example.com", HttpMethod::GET),
        };
        assert!(!config.is_retryable(&error));
    }

    #[test]
    fn test_retry_executor_success() {
        let config = RetryConfig::new().max_retries(3).jitter(false);
        let executor = RetryExecutor::new(&config);

        let result = executor.execute(|| Ok::<_, HttpError>(42));
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_retry_executor_with_result() {
        let config = RetryConfig::new().max_retries(3).jitter(false);
        let executor = RetryExecutor::new(&config);

        let result = executor.execute_with_result(|| Ok::<_, HttpError>(42)).unwrap();
        assert_eq!(result.result, 42);
        assert_eq!(result.retry_count, 0);
        assert!(result.succeeded_on_first_attempt);
        assert!(result.total_duration.as_millis() < 100);
    }

    #[test]
    fn test_retry_executor_retry_then_success() {
        let config = RetryConfig::new()
            .max_retries(3)
            .initial_delay(Duration::from_millis(1))
            .jitter(false);
        let executor = RetryExecutor::new(&config);

        let mut attempts = 0;
        let result = executor
            .execute_with_result(|| {
                attempts += 1;
                if attempts < 3 {
                    Err(HttpError::Timeout {
                        context: ErrorContext::boxed("https://example.com", HttpMethod::GET),
                    })
                } else {
                    Ok(42)
                }
            })
            .unwrap();

        assert_eq!(result.result, 42);
        assert_eq!(result.retry_count, 2);
        assert!(!result.succeeded_on_first_attempt);
        assert_eq!(attempts, 3);
    }

    #[test]
    fn test_retry_executor_exhausted() {
        let config = RetryConfig::new()
            .max_retries(2)
            .initial_delay(Duration::from_millis(1))
            .jitter(false);
        let executor = RetryExecutor::new(&config);

        let result: Result<i32, _> = executor.execute(|| {
            Err(HttpError::Timeout {
                context: ErrorContext::boxed("https://example.com", HttpMethod::GET),
            })
        });

        assert!(matches!(
            result,
            Err(HttpError::RetryExhausted { attempts: 3, .. })
        ));
    }

    #[test]
    fn test_retry_result_into_result() {
        let retry_result = RetryResult {
            result: 42,
            retry_count: 1,
            succeeded_on_first_attempt: false,
            total_duration: Duration::from_millis(100),
        };

        assert_eq!(retry_result.into_result(), 42);
    }
}
