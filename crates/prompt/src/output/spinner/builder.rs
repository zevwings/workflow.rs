//! Spinner 构建器
//!
//! 使用构建器模式创建和配置 Spinner。

use crate::output::spinner::spinner::Spinner;
use std::time::Duration;

/// Spinner 构建器
pub struct SpinnerBuilder {
    message: String,
    frames: Option<Vec<String>>,
    interval: Option<Duration>,
}

impl SpinnerBuilder {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            frames: None,
            interval: None,
        }
    }

    pub fn with_frames(mut self, frames: Vec<impl Into<String>>) -> Self {
        self.frames = Some(frames.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = Some(interval);
        self
    }

    /// 使用 spinner 执行一个操作（便捷方法）
    pub fn with<F, T, E>(self, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let message_str = self.message.clone();
        let spinner = self.start();
        let start = std::time::Instant::now();
        let result = operation();
        let elapsed = start.elapsed();

        if elapsed < Duration::from_millis(100) {
            spinner.finish_with_message(message_str);
        } else {
            spinner.stop();
        }

        result
    }

    /// 使用 spinner 执行一个会产生输出的操作
    pub fn with_output<F, T, E>(self, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let spinner = self.start();
        std::thread::sleep(Duration::from_millis(250));
        spinner.stop();
        operation()
    }

    pub fn start(self) -> Spinner {
        let default_frames = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
            .into_iter()
            .map(String::from)
            .collect();

        let spinner = Spinner::new_internal(
            self.message,
            self.frames.unwrap_or(default_frames),
            self.interval.unwrap_or(Duration::from_millis(100)),
        );

        spinner.start_internal();
        spinner
    }
}
