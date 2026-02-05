//! 进度条构建器
//!
//! 使用构建器模式创建和配置进度条。

use crate::output::progress::bar::{ProgressBar, ProgressMode};
use std::time::Duration;

/// 进度条构建器
///
/// 使用构建器模式创建和配置进度条。
///
/// # 示例
///
/// ```rust,no_run
/// use prompt::progress_bar;
/// use std::time::Duration;
///
/// let pb = progress_bar("Downloading...")
///     .with_total(1024 * 1024)
///     .with_interval(Duration::from_millis(50))
///     .with_bar_width(40)
///     .with_progress_chars("█░")
///     .start();
/// ```
pub struct ProgressBarBuilder {
    message: String,
    total: Option<u64>,
    interval: Option<Duration>,
    bar_width: Option<usize>,
    progress_chars: Option<String>,
    mode: ProgressMode,
}

impl ProgressBarBuilder {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            total: None,
            interval: None,
            bar_width: None,
            progress_chars: None,
            mode: ProgressMode::Normal,
        }
    }

    /// 设置总长度（已知总数）
    pub fn with_total(mut self, total: u64) -> Self {
        self.total = Some(total);
        self
    }

    /// 设置刷新间隔
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = Some(interval);
        self
    }

    /// 设置进度条宽度
    pub fn with_bar_width(mut self, width: usize) -> Self {
        self.bar_width = Some(width);
        self
    }

    /// 设置进度条字符（如 "█░" 或 "#>-"）
    pub fn with_progress_chars(mut self, chars: impl Into<String>) -> Self {
        self.progress_chars = Some(chars.into());
        self
    }

    /// 设置为下载模式（显示字节数、速度、ETA）
    pub fn with_download_mode(mut self) -> Self {
        self.mode = ProgressMode::Download;
        self
    }

    /// 启动进度条
    pub fn start(self) -> ProgressBar {
        let progress_bar = ProgressBar::new_internal(
            self.message,
            self.total,
            self.mode,
            self.interval.unwrap_or(Duration::from_millis(100)),
            self.bar_width.unwrap_or(30),
            self.progress_chars.unwrap_or_else(|| "█░".to_string()),
        );

        progress_bar.start_internal();
        progress_bar
    }

    /// 创建进度条但不启动（用于测试）
    #[cfg(any(test, feature = "testing"))]
    pub fn build_without_start(self) -> ProgressBar {
        ProgressBar::new_internal(
            self.message,
            self.total,
            self.mode,
            self.interval.unwrap_or(Duration::from_millis(100)),
            self.bar_width.unwrap_or(30),
            self.progress_chars.unwrap_or_else(|| "█░".to_string()),
        )
    }

    /// 获取配置的消息（用于测试）
    #[cfg(any(test, feature = "testing"))]
    pub fn get_message(&self) -> &str {
        &self.message
    }

    /// 获取配置的总数（用于测试）
    #[cfg(any(test, feature = "testing"))]
    pub fn get_total(&self) -> Option<u64> {
        self.total
    }

    /// 获取配置的模式（用于测试）
    #[cfg(any(test, feature = "testing"))]
    #[allow(dead_code)]
    pub(crate) fn get_mode(&self) -> ProgressMode {
        self.mode
    }

    /// 获取配置的进度条宽度（用于测试）
    #[cfg(any(test, feature = "testing"))]
    pub fn get_bar_width(&self) -> Option<usize> {
        self.bar_width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_builder_new() {
        let builder = ProgressBarBuilder::new("Loading...");
        assert_eq!(builder.get_message(), "Loading...");
        assert!(builder.get_total().is_none());
        assert!(matches!(builder.get_mode(), ProgressMode::Normal));
    }

    #[test]
    fn test_progress_builder_with_total() {
        let builder = ProgressBarBuilder::new("Downloading").with_total(1000);
        assert_eq!(builder.get_total(), Some(1000));
    }

    #[test]
    fn test_progress_builder_with_interval() {
        let builder =
            ProgressBarBuilder::new("Processing").with_interval(Duration::from_millis(50));
        assert_eq!(builder.interval, Some(Duration::from_millis(50)));
    }

    #[test]
    fn test_progress_builder_with_bar_width() {
        let builder = ProgressBarBuilder::new("Working").with_bar_width(50);
        assert_eq!(builder.get_bar_width(), Some(50));
    }

    #[test]
    fn test_progress_builder_with_progress_chars() {
        let builder = ProgressBarBuilder::new("Loading").with_progress_chars("#>-");
        assert_eq!(builder.progress_chars, Some("#>-".to_string()));
    }

    #[test]
    fn test_progress_builder_with_download_mode() {
        let builder = ProgressBarBuilder::new("Downloading").with_download_mode();
        assert!(matches!(builder.get_mode(), ProgressMode::Download));
    }

    #[test]
    fn test_progress_builder_chain() {
        let builder = ProgressBarBuilder::new("Downloading files")
            .with_total(1024 * 1024)
            .with_interval(Duration::from_millis(100))
            .with_bar_width(40)
            .with_progress_chars("█▓▒░")
            .with_download_mode();

        assert_eq!(builder.get_message(), "Downloading files");
        assert_eq!(builder.get_total(), Some(1024 * 1024));
        assert!(matches!(builder.get_mode(), ProgressMode::Download));
        assert_eq!(builder.get_bar_width(), Some(40));
    }

    #[test]
    fn test_progress_builder_build_without_start() {
        let pb = ProgressBarBuilder::new("Test").with_total(100).build_without_start();

        // 进度条应该被创建但没有运行
        drop(pb);
    }

    #[test]
    fn test_progress_builder_unicode_message() {
        let builder = ProgressBarBuilder::new("正在下载...");
        assert_eq!(builder.get_message(), "正在下载...");
    }

    #[test]
    fn test_progress_builder_empty_message() {
        let builder = ProgressBarBuilder::new("");
        assert_eq!(builder.get_message(), "");
    }

    #[test]
    fn test_progress_builder_large_total() {
        let builder = ProgressBarBuilder::new("Large file").with_total(u64::MAX);
        assert_eq!(builder.get_total(), Some(u64::MAX));
    }
}
