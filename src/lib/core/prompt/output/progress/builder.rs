//! 进度条构建器
//!
//! 使用构建器模式创建和配置进度条。

use super::bar::{ProgressBar, ProgressMode};
use std::time::Duration;

/// 进度条构建器
///
/// 使用构建器模式创建和配置进度条。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::prompt::progress_bar;
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
}
