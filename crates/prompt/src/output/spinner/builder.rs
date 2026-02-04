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

    /// 创建 Spinner 但不启动（用于测试）
    #[cfg(any(test, feature = "testing"))]
    pub fn build_without_start(self) -> Spinner {
        let default_frames = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
            .into_iter()
            .map(String::from)
            .collect();

        Spinner::new_internal(
            self.message,
            self.frames.unwrap_or(default_frames),
            self.interval.unwrap_or(Duration::from_millis(100)),
        )
    }

    /// 获取配置的消息（用于测试）
    #[cfg(any(test, feature = "testing"))]
    pub fn get_message(&self) -> &str {
        &self.message
    }

    /// 获取配置的帧（用于测试）
    #[cfg(any(test, feature = "testing"))]
    pub fn get_frames(&self) -> Option<&Vec<String>> {
        self.frames.as_ref()
    }

    /// 获取配置的间隔（用于测试）
    #[cfg(any(test, feature = "testing"))]
    pub fn get_interval(&self) -> Option<Duration> {
        self.interval
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_builder_new() {
        let builder = SpinnerBuilder::new("Loading...");
        assert_eq!(builder.get_message(), "Loading...");
        assert!(builder.get_frames().is_none());
        assert!(builder.get_interval().is_none());
    }

    #[test]
    fn test_spinner_builder_with_frames() {
        let frames = vec![".", "..", "...", "...."];
        let builder = SpinnerBuilder::new("Loading").with_frames(frames.clone());

        let configured_frames = builder.get_frames().unwrap();
        assert_eq!(configured_frames.len(), 4);
        assert_eq!(configured_frames[0], ".");
        assert_eq!(configured_frames[3], "....");
    }

    #[test]
    fn test_spinner_builder_with_interval() {
        let builder = SpinnerBuilder::new("Loading").with_interval(Duration::from_millis(200));

        assert_eq!(builder.get_interval(), Some(Duration::from_millis(200)));
    }

    #[test]
    fn test_spinner_builder_chain() {
        let builder = SpinnerBuilder::new("Processing")
            .with_frames(vec!["⠋", "⠙", "⠹"])
            .with_interval(Duration::from_millis(50));

        assert_eq!(builder.get_message(), "Processing");
        assert!(builder.get_frames().is_some());
        assert_eq!(builder.get_interval(), Some(Duration::from_millis(50)));
    }

    #[test]
    fn test_spinner_builder_build_without_start() {
        let spinner = SpinnerBuilder::new("Test").build_without_start();

        // Spinner 应该被创建但没有运行
        // 我们无法直接检查内部状态，但可以确保不会崩溃
        drop(spinner);
    }

    #[test]
    fn test_spinner_builder_with_custom_message() {
        let builder = SpinnerBuilder::new("Downloading files...");
        assert_eq!(builder.get_message(), "Downloading files...");
    }

    #[test]
    fn test_spinner_builder_with_unicode_message() {
        let builder = SpinnerBuilder::new("正在加载...");
        assert_eq!(builder.get_message(), "正在加载...");
    }

    #[test]
    fn test_spinner_builder_with_empty_message() {
        let builder = SpinnerBuilder::new("");
        assert_eq!(builder.get_message(), "");
    }

    #[test]
    fn test_spinner_builder_default_frames() {
        let builder = SpinnerBuilder::new("Loading");
        // 默认应该没有自定义帧
        assert!(builder.get_frames().is_none());

        // 使用 build_without_start 创建时会使用默认帧
        let _spinner = builder.build_without_start();
    }

    #[test]
    fn test_spinner_builder_with_single_frame() {
        let builder = SpinnerBuilder::new("Single").with_frames(vec!["*"]);

        let frames = builder.get_frames().unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0], "*");
    }

    #[test]
    fn test_spinner_builder_with_unicode_frames() {
        let frames = vec!["🌑", "🌒", "🌓", "🌔", "🌕"];
        let builder = SpinnerBuilder::new("Moon").with_frames(frames.clone());

        let configured = builder.get_frames().unwrap();
        assert_eq!(configured.len(), 5);
        assert_eq!(configured[0], "🌑");
        assert_eq!(configured[4], "🌕");
    }

    #[test]
    fn test_spinner_builder_with_very_short_interval() {
        let builder = SpinnerBuilder::new("Fast").with_interval(Duration::from_millis(1));

        assert_eq!(builder.get_interval(), Some(Duration::from_millis(1)));
    }

    #[test]
    fn test_spinner_builder_with_long_interval() {
        let builder = SpinnerBuilder::new("Slow").with_interval(Duration::from_secs(5));

        assert_eq!(builder.get_interval(), Some(Duration::from_secs(5)));
    }

    #[test]
    fn test_spinner_builder_build_without_start_custom_config() {
        let spinner = SpinnerBuilder::new("Custom")
            .with_frames(vec!["-", "\\", "|", "/"])
            .with_interval(Duration::from_millis(50))
            .build_without_start();

        // 验证 spinner 被正确创建
        drop(spinner);
    }

    #[test]
    fn test_spinner_builder_message_ownership() {
        let message = String::from("Dynamic message");
        let builder = SpinnerBuilder::new(message);
        assert_eq!(builder.get_message(), "Dynamic message");
    }

    #[test]
    fn test_spinner_builder_frames_converted_to_string() {
        // 测试 &str 到 String 的转换
        let builder = SpinnerBuilder::new("Test").with_frames(vec!["a", "b", "c"]);

        let frames = builder.get_frames().unwrap();
        assert_eq!(frames[0], "a");
    }

    #[test]
    fn test_spinner_builder_multiple_reconfigurations() {
        let builder = SpinnerBuilder::new("Initial")
            .with_interval(Duration::from_millis(100))
            .with_frames(vec!["1", "2"])
            .with_interval(Duration::from_millis(200)); // 覆盖之前的设置

        assert_eq!(builder.get_interval(), Some(Duration::from_millis(200)));
    }

    // 注意：以下测试会实际启动 spinner，可能在 CI 环境中需要特殊处理

    #[test]
    fn test_spinner_builder_with_success_operation() {
        // 测试 with 方法的成功路径
        let result: Result<i32, &str> = SpinnerBuilder::new("Computing")
            .with_interval(Duration::from_millis(10))
            .with(|| Ok(42));

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_spinner_builder_with_error_operation() {
        // 测试 with 方法的错误路径
        let result: Result<i32, &str> = SpinnerBuilder::new("Computing")
            .with_interval(Duration::from_millis(10))
            .with(|| Err("failed"));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "failed");
    }

    #[test]
    fn test_spinner_builder_with_output_success() {
        // 测试 with_output 方法
        let result: Result<String, &str> = SpinnerBuilder::new("Processing")
            .with_interval(Duration::from_millis(10))
            .with_output(|| Ok("output".to_string()));

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "output");
    }

    #[test]
    fn test_spinner_builder_with_output_error() {
        let result: Result<(), &str> = SpinnerBuilder::new("Processing")
            .with_interval(Duration::from_millis(10))
            .with_output(|| Err("error"));

        assert!(result.is_err());
    }
}
