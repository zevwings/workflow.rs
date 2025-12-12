//! Spinner 工具模块
//!
//! 提供统一的 loading spinner 功能，用于显示长时间运行的操作进度。

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::io::{self, Write};
use std::time::Duration;

/// Spinner 结构体
///
/// 用于显示长时间运行操作的进度指示器。
///
/// # 示例
///
/// ```rust
/// use workflow::base::indicator::Spinner;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // 方式 1：手动管理
/// let spinner = Spinner::new("Creating PR...");
/// // 执行耗时操作
/// spinner.finish();
///
/// // 方式 2：自动管理（推荐）
/// let result: Result<i32, Box<dyn std::error::Error>> = Spinner::with("Creating PR...", || {
///     // 执行操作
///     Ok(42)
/// });
/// let _ = result?;
/// # Ok(())
/// # }
/// ```
pub struct Spinner {
    inner: ProgressBar,
}

impl Drop for Spinner {
    fn drop(&mut self) {
        // 确保 Spinner 在析构时总是被清理
        // 这可以防止在错误或中断情况下 Spinner 没有被正确清理
        self.inner.finish_and_clear();
        // 刷新 stderr 和 stdout，确保 Spinner 清除操作立即生效
        let _ = io::stderr().flush();
        let _ = io::stdout().flush();
    }
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
    /// use workflow::base::indicator::Spinner;
    ///
    /// let spinner = Spinner::new("Creating PR...");
    /// // 执行耗时操作
    /// spinner.finish();
    /// ```
    pub fn new(message: impl AsRef<str>) -> Self {
        let spinner = ProgressBar::new_spinner();
        // 将 Spinner 输出重定向到 stderr，避免与 stdout 的日志输出冲突
        spinner.set_draw_target(ProgressDrawTarget::stderr());
        spinner.set_style(
            ProgressStyle::default_spinner().template("{spinner:.white} {msg}").unwrap(),
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
    /// use workflow::base::indicator::Spinner;
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
    /// use workflow::base::indicator::Spinner;
    ///
    /// let spinner = Spinner::new("Creating PR...");
    /// // 执行操作
    /// spinner.finish();
    /// ```
    pub fn finish(self) {
        self.inner.finish_and_clear();
        // 刷新 stderr 和 stdout，确保 Spinner 清除操作立即生效
        // 这样可以避免后续输出与 Spinner 冲突
        let _ = io::stderr().flush();
        let _ = io::stdout().flush();
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
    /// use workflow::base::indicator::Spinner;
    ///
    /// let spinner = Spinner::new("Creating PR...");
    /// // 执行操作
    /// spinner.finish_with_message("PR created successfully!");
    /// ```
    pub fn finish_with_message(self, message: impl AsRef<str>) {
        self.inner.finish_with_message(message.as_ref().to_string());
        // 刷新 stderr 和 stdout，确保 Spinner 清除操作立即生效
        // 这样可以避免后续输出与 Spinner 冲突
        let _ = io::stderr().flush();
        let _ = io::stdout().flush();
    }

    /// 使用 spinner 执行一个操作（便捷方法）
    ///
    /// 自动创建 spinner，执行操作，然后清理 spinner。
    /// 如果操作很快完成（< 100ms），会使用 `finish_with_message` 显示完成消息，
    /// 确保用户至少能看到一次输出。
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
    /// use workflow::base::indicator::Spinner;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result: Result<i32, Box<dyn std::error::Error>> = Spinner::with("Creating PR...", || {
    ///     // 执行操作
    ///     Ok(42)
    /// });
    /// let _ = result?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with<F, T, E>(message: impl AsRef<str>, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let message_str = message.as_ref().to_string();
        let spinner = Self::new(&message_str);
        let start = std::time::Instant::now();
        let result = operation();
        let elapsed = start.elapsed();

        // 如果操作很快完成（< 100ms），使用 finish_with_message 显示消息
        // 确保用户至少能看到一次输出
        if elapsed < Duration::from_millis(100) {
            spinner.finish_with_message(&message_str);
        } else {
            spinner.finish();
        }

        result
    }

    /// 使用 spinner 执行一个会产生输出的操作
    ///
    /// 先显示 spinner 消息（250ms），然后完成 spinner，再执行操作。
    /// 这样可以确保用户能看到消息，同时让子进程的输出正常显示。
    ///
    /// 这个方法适用于执行会产生 stdout/stderr 输出的操作（如 `git push`），
    /// 可以避免子进程的输出与 spinner 动画混合。
    ///
    /// **注意**：操作完成后，建议使用 `log_info!` 或 `log_success!` 显示完成状态。
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
    /// ```rust,no_run
    /// use workflow::base::indicator::Spinner;
    /// use workflow::log_success;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result: Result<(), Box<dyn std::error::Error>> = Spinner::with_output("Pushing to remote...", || {
    ///     // 执行操作
    ///     Ok(())
    /// });
    /// result?;
    /// log_success!("Pushed to remote successfully");
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_output<F, T, E>(message: impl AsRef<str>, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let spinner = Self::new(message);
        // 让 spinner 显示足够长的时间（250ms），确保用户能看到消息
        std::thread::sleep(Duration::from_millis(250));
        // 完成 spinner（清除它），然后执行操作
        spinner.finish();
        // 执行操作，让子进程的输出正常显示
        operation()
    }
}
