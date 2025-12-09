//! Spinner 工具模块
//!
//! 提供统一的 loading spinner 功能，用于显示长时间运行的操作进度。

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Spinner 结构体
///
/// 用于显示长时间运行操作的进度指示器。
///
/// # 示例
///
/// ```rust
/// use crate::base::indicator::Spinner;
///
/// // 方式 1：手动管理
/// let spinner = Spinner::new("Creating PR...");
/// // 执行耗时操作
/// spinner.finish();
///
/// // 方式 2：自动管理（推荐）
/// let result = Spinner::with("Creating PR...", || {
///     provider.create_pull_request(...)
/// })?;
/// ```
pub struct Spinner {
    inner: ProgressBar,
}

impl Spinner {
    /// 创建一个新的 spinner 并立即开始显示
    ///
    /// # 参数
    ///
    /// * `message` - 要显示的消息文本
    ///
    /// # 返回
    ///
    /// 返回配置好的 `Spinner` 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::base::indicator::Spinner;
    ///
    /// let spinner = Spinner::new("Creating PR...");
    /// // 执行耗时操作
    /// spinner.finish();
    /// ```
    pub fn new(message: impl AsRef<str>) -> Self {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.white} {msg}")
                .unwrap(),
        );
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_message(message.as_ref().to_string());

        Self { inner: spinner }
    }

    /// 更新 spinner 显示的消息
    ///
    /// # 参数
    ///
    /// * `message` - 新的消息文本
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::base::indicator::Spinner;
    ///
    /// let spinner = Spinner::new("Starting...");
    /// spinner.update_message("Processing...");
    /// spinner.finish();
    /// ```
    pub fn update_message(&self, message: impl AsRef<str>) {
        self.inner.set_message(message.as_ref().to_string());
    }

    /// 完成并清除 spinner
    ///
    /// 停止 spinner 动画并清除显示。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::base::indicator::Spinner;
    ///
    /// let spinner = Spinner::new("Creating PR...");
    /// // 执行操作
    /// spinner.finish();
    /// ```
    pub fn finish(self) {
        self.inner.finish_and_clear();
    }

    /// 完成 spinner 并显示完成消息
    ///
    /// 停止 spinner 动画并显示完成消息，然后清除。
    ///
    /// # 参数
    ///
    /// * `message` - 完成消息
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::base::indicator::Spinner;
    ///
    /// let spinner = Spinner::new("Creating PR...");
    /// // 执行操作
    /// spinner.finish_with_message("PR created successfully!");
    /// ```
    pub fn finish_with_message(self, message: impl AsRef<str>) {
        self.inner.finish_with_message(message.as_ref().to_string());
    }

    /// 使用 spinner 执行一个操作（便捷方法）
    ///
    /// 自动创建 spinner，执行操作，然后清理 spinner。
    ///
    /// # 参数
    ///
    /// * `message` - 要显示的消息文本
    /// * `operation` - 要执行的操作（闭包）
    ///
    /// # 返回
    ///
    /// 返回操作的结果
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::base::indicator::Spinner;
    ///
    /// let result = Spinner::with("Creating PR...", || {
    ///     provider.create_pull_request(...)
    /// })?;
    /// ```
    pub fn with<F, T, E>(message: impl AsRef<str>, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let spinner = Self::new(message);
        let result = operation();
        spinner.finish();
        result
    }
}
