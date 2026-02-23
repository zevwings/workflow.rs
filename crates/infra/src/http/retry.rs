//! HTTP 重试配置和逻辑

use std::{thread, time::Duration};

use client::HttpError;
use rand::Rng;
use toolkit::log_debug;

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
    /// 是否启用 jitter
    pub jitter: bool,
    /// Jitter 因子（0.0 - 1.0）
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

    /// 检查错误是否可重试
    pub fn is_retryable(&self, error: &HttpError) -> bool {
        match error {
            HttpError::Timeout { .. } => true,
            HttpError::Connection { .. } => true,
            HttpError::Status { status, .. } => self.retryable_status_codes.contains(status),
            _ => false,
        }
    }

    /// 计算指定重试次数的延迟
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_delay = if attempt == 0 {
            self.initial_delay
        } else {
            let delay_ms =
                self.initial_delay.as_millis() as f64 * self.backoff_factor.powi(attempt as i32);
            let delay = Duration::from_millis(delay_ms as u64);
            if delay > self.max_delay {
                self.max_delay
            } else {
                delay
            }
        };

        if !self.jitter || self.jitter_factor == 0.0 {
            return base_delay;
        }

        let jitter_range = self.jitter_factor * 2.0;
        let mut rng = rand::rng();
        let jitter_multiplier = 1.0 - self.jitter_factor + rng.random::<f64>() * jitter_range;
        let jittered_ms = base_delay.as_millis() as f64 * jitter_multiplier;
        Duration::from_millis(jittered_ms.max(1.0) as u64)
    }
}

/// 执行带重试的操作
pub fn execute_with_retry<F, T>(config: &RetryConfig, mut operation: F) -> Result<T, HttpError>
where
    F: FnMut() -> Result<T, HttpError>,
{
    let mut attempt = 0;

    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempt >= config.max_retries || !config.is_retryable(&error) {
                    if attempt > 0 {
                        return Err(HttpError::RetryExhausted {
                            attempts: attempt + 1,
                            last_error: Box::new(error),
                        });
                    }
                    return Err(error);
                }

                let delay = config.delay_for_attempt(attempt);
                log_debug!(
                    attempt = attempt + 1,
                    max_retries = config.max_retries,
                    delay_ms = delay.as_millis(),
                    error = %error,
                    "Retrying request"
                );

                thread::sleep(delay);
                attempt += 1;
            }
        }
    }
}
