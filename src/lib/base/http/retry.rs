//! HTTP 重试工具模块
//!
//! 本模块提供了专门用于 HTTP 请求的重试机制，支持指数退避算法。
//! 针对 HTTP 请求的错误类型进行智能判断，自动重试可恢复的错误。
//! 支持用户交互：在重试前询问用户是否继续，允许用户取消操作。

use crate::{log_debug, log_success, log_warning};
use anyhow::Result;
use dialoguer::Confirm;

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
    /// 是否启用交互式确认（默认：true）
    /// 如果为 true，在重试前会询问用户是否继续
    pub interactive: bool,
}

impl Default for HttpRetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: 1,
            max_delay: 30,
            backoff_multiplier: 2.0,
            interactive: true,
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
    /// - `backoff_multiplier`: 2.0
    pub fn new() -> Self {
        Self::default()
    }
}

/// HTTP 重试工具
///
/// 提供 HTTP 请求重试的功能，支持指数退避算法和用户交互。
/// 专门针对 HTTP 请求的错误类型进行智能判断。
pub struct HttpRetry;

impl HttpRetry {
    /// 使用指数退避算法重试 HTTP 操作
    ///
    /// 执行一个可能失败的 HTTP 操作，如果失败且错误可重试，则按照配置的重试策略进行重试。
    /// 使用指数退避算法，每次重试的延迟时间会逐渐增加，直到达到最大延迟。
    ///
    /// # 参数
    ///
    /// * `operation` - 要执行的操作（闭包）
    /// * `config` - 重试配置
    /// * `operation_name` - 操作名称（用于日志输出）
    ///
    /// # 类型参数
    ///
    /// * `F` - 操作闭包类型，必须返回 `Result<T>`
    /// * `T` - 操作返回值的类型
    ///
    /// # 返回
    ///
    /// 返回操作的结果。如果所有重试都失败，返回最后一次的错误。
    ///
    /// # 错误处理
    ///
    /// - 如果错误不可重试（如 4xx 客户端错误），立即返回错误，不进行重试
    /// - 如果所有重试都失败，返回最后一次的错误，并附加重试信息
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::base::http::retry::{HttpRetry, HttpRetryConfig};
    ///
    /// let config = HttpRetryConfig::new();
    /// let result = HttpRetry::retry(
    ///     || {
    ///         // 执行可能失败的 HTTP 操作
    ///         Ok(42)
    ///     },
    ///     &config,
    ///     "获取数据",
    /// )?;
    /// ```
    pub fn retry<F, T>(operation: F, config: &HttpRetryConfig, operation_name: &str) -> Result<T>
    where
        F: Fn() -> Result<T>,
    {
        let mut delay = config.initial_delay;
        let mut last_error = None;

        for attempt in 0..=config.max_retries {
            match operation() {
                Ok(result) => {
                    if attempt > 0 {
                        log_success!(
                            "{} succeeded after {} retry attempts",
                            operation_name,
                            attempt
                        );
                    }
                    return Ok(result);
                }
                Err(e) => {
                    let error = e;
                    let error_desc = Self::get_error_description(&error);
                    last_error = Some(error);

                    // 检查是否可重试
                    if let Some(ref err) = last_error {
                        if !Self::is_retryable_error(err) {
                            // 错误不可重试，立即返回
                            if attempt == 0 {
                                // 第一次尝试就失败，且不可重试
                                log_warning!(
                                    "{} failed: {} (not retryable)",
                                    operation_name,
                                    error_desc
                                );
                            }
                            return Err(last_error.unwrap());
                        }
                    }

                    // 如果还有重试机会
                    if attempt < config.max_retries {
                        log_warning!(
                            "{} failed: {} (attempt {}/{})",
                            operation_name,
                            error_desc,
                            attempt + 1,
                            config.max_retries + 1
                        );

                        // 交互式确认：询问用户是否继续重试
                        if config.interactive && attempt > 0 {
                            let prompt = format!(
                                "是否在 {} 秒后重试？(尝试 {}/{})",
                                delay,
                                attempt + 1,
                                config.max_retries + 1
                            );
                            match Confirm::new().with_prompt(&prompt).default(true).interact() {
                                Ok(true) => {
                                    // 用户选择继续，显示倒计时
                                    Self::countdown_with_cancel(delay, operation_name)?;
                                }
                                Ok(false) => {
                                    // 用户选择取消
                                    log_warning!("User cancelled operation");
                                    return Err(anyhow::anyhow!("User cancelled operation"));
                                }
                                Err(e) => {
                                    // 交互失败，可能是非交互式终端，直接继续
                                    log_debug!("Failed to get user input, auto-continuing: {}", e);
                                    std::thread::sleep(std::time::Duration::from_secs(delay));
                                }
                            }
                        } else {
                            // 非交互模式或第一次重试，直接等待
                            if config.interactive && attempt == 0 {
                                // 第一次失败，显示倒计时
                                Self::countdown_with_cancel(delay, operation_name)?;
                            } else {
                                std::thread::sleep(std::time::Duration::from_secs(delay));
                            }
                        }

                        delay = ((delay as f64 * config.backoff_multiplier) as u64)
                            .min(config.max_delay);
                    } else {
                        // 所有重试都失败了
                        log_warning!(
                            "{} failed: {} (retried {} times)",
                            operation_name,
                            error_desc,
                            config.max_retries
                        );
                    }
                }
            }
        }

        // 所有重试都失败，返回最后一次的错误，并添加上下文信息
        let final_error = last_error.unwrap();
        Err(final_error.context(format!(
            "{} failed after {} retries",
            operation_name, config.max_retries
        )))
    }

    /// 判断错误是否可重试
    ///
    /// 检查错误类型，判断是否应该重试。
    /// 可重试的错误包括：
    /// - 网络错误（超时、连接失败、请求中断）
    /// - 5xx 服务器错误（500, 502, 503, 504）
    /// - 429 Too Many Requests（需要特殊处理，使用 Retry-After header）
    ///
    /// 不可重试的错误包括：
    /// - 4xx 客户端错误（400, 401, 403, 404 等）
    /// - 解析错误（JSON 解析失败、文件格式错误）
    /// - 其他非网络错误
    ///
    /// # 参数
    ///
    /// * `error` - 要检查的错误
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果错误可重试，否则返回 `false`。
    fn is_retryable_error(error: &anyhow::Error) -> bool {
        // 检查是否是 reqwest 网络错误
        if let Some(reqwest_error) = error.downcast_ref::<reqwest::Error>() {
            // 检查是否是网络连接错误
            if reqwest_error.is_timeout()
                || reqwest_error.is_connect()
                || reqwest_error.is_request()
            {
                return true;
            }

            // 检查 HTTP 状态码
            if let Some(status) = reqwest_error.status() {
                // 5xx 服务器错误和 429 Too Many Requests 可重试
                return status.is_server_error() || status.as_u16() == 429;
            }
        }

        // 检查是否是标准库 IO 错误（可能是网络相关的）
        if let Some(io_error) = error.downcast_ref::<std::io::Error>() {
            // 网络相关的 IO 错误可重试
            match io_error.kind() {
                std::io::ErrorKind::TimedOut
                | std::io::ErrorKind::ConnectionRefused
                | std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::ConnectionAborted
                | std::io::ErrorKind::NotConnected
                | std::io::ErrorKind::BrokenPipe => return true,
                _ => {}
            }
        }

        false
    }

    /// 倒计时等待，支持用户取消
    ///
    /// 在等待期间每秒更新一次倒计时，用户可以通过 Ctrl+C 取消。
    fn countdown_with_cancel(seconds: u64, operation_name: &str) -> Result<()> {
        use std::io::{self, Write};
        use std::time::{Duration, Instant};

        let start = Instant::now();
        let duration = Duration::from_secs(seconds);

        // 如果等待时间很短（小于 3 秒），直接等待
        if seconds < 3 {
            std::thread::sleep(duration);
            return Ok(());
        }

        // 显示倒计时
        let mut remaining = seconds;
        while remaining > 0 {
            // 检查是否已经等待足够的时间
            if start.elapsed() >= duration {
                break;
            }

            // 显示倒计时（每 2 秒更新一次，避免输出过多）
            if remaining.is_multiple_of(2) || remaining <= 3 {
                print!(
                    "\r  {} 秒后重试 {}... (按 Ctrl+C 取消)  ",
                    remaining, operation_name
                );
                io::stdout().flush().ok();
            }

            // 等待 1 秒
            std::thread::sleep(Duration::from_secs(1));
            remaining = remaining.saturating_sub(1);
        }

        // 清除倒计时行
        print!("\r{}", " ".repeat(60));
        print!("\r");
        io::stdout().flush().ok();

        Ok(())
    }

    /// 从错误中提取可读的错误描述
    ///
    /// 尝试从错误中提取有用的信息，用于日志输出。
    ///
    /// # 参数
    ///
    /// * `error` - 要提取描述的错误
    ///
    /// # 返回
    ///
    /// 返回错误的简短描述。
    fn get_error_description(error: &anyhow::Error) -> String {
        // 尝试从 reqwest 错误中提取状态码
        if let Some(reqwest_error) = error.downcast_ref::<reqwest::Error>() {
            if let Some(status) = reqwest_error.status() {
                return format!("HTTP {}", status.as_u16());
            }
            if reqwest_error.is_timeout() {
                return "Network timeout".to_string();
            }
            if reqwest_error.is_connect() {
                return "Connection failed".to_string();
            }
        }

        // 尝试从 IO 错误中提取信息
        if let Some(io_error) = error.downcast_ref::<std::io::Error>() {
            return format!("IO error: {}", io_error.kind());
        }

        // 默认返回错误消息的前 100 个字符
        let error_msg = error.to_string();
        if error_msg.len() > 100 {
            format!("{}...", &error_msg[..100])
        } else {
            error_msg
        }
    }
}
