//! Progress Bar 工具模块
//!
//! 提供统一的进度条功能，用于显示有明确进度的操作（如下载、上传等）。

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Progress Bar 结构体
///
/// 用于显示有明确进度的操作（如下载文件、处理多个项目等）。
///
/// # 示例
///
/// ```rust
/// use workflow::base::indicator::Progress;
///
/// // 方式 1：已知总数
/// let progress = Progress::new(100, "Downloading files...");
/// for i in 0..100 {
///     // 处理项目
///     progress.inc(1);
/// }
/// progress.finish();
///
/// // 方式 2：未知总数（使用 spinner 模式）
/// let progress = Progress::new_unknown("Downloading...");
/// // 执行操作
/// progress.finish();
/// ```
pub struct Progress {
    inner: ProgressBar,
}

impl Progress {
    /// 创建一个新的进度条（已知总数）
    ///
    /// # 参数
    ///
    /// * `total` - 总数量（文件数、字节数等）
    /// * `message` - 要显示的消息文本
    ///
    /// # 返回
    ///
    /// 返回配置好的 `Progress` 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use workflow::base::indicator::Progress;
    ///
    /// let progress = Progress::new(100, "Downloading files...");
    /// progress.inc(1);
    /// progress.finish();
    /// ```
    pub fn new(total: u64, message: impl AsRef<str>) -> Self {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(message.as_ref().to_string());
        pb.enable_steady_tick(Duration::from_millis(100));

        Self { inner: pb }
    }

    /// 创建一个新的进度条（用于下载，显示字节数）
    ///
    /// # 参数
    ///
    /// * `total_bytes` - 总字节数
    /// * `message` - 要显示的消息文本
    ///
    /// # 返回
    ///
    /// 返回配置好的 `Progress` 实例，显示下载进度
    ///
    /// # 示例
    ///
    /// ```rust
    /// use workflow::base::indicator::Progress;
    ///
    /// let progress = Progress::new_download(1024 * 1024, "Downloading...");
    /// progress.inc_bytes(1024);
    /// progress.finish();
    /// ```
    pub fn new_download(total_bytes: u64, message: impl AsRef<str>) -> Self {
        let pb = ProgressBar::new(total_bytes);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(message.as_ref().to_string());
        pb.enable_steady_tick(Duration::from_millis(100));

        Self { inner: pb }
    }

    /// 创建一个新的进度条（未知总数，使用 spinner 模式）
    ///
    /// 当无法确定总数时使用，会显示一个 spinner 和当前进度。
    ///
    /// # 参数
    ///
    /// * `message` - 要显示的消息文本
    ///
    /// # 返回
    ///
    /// 返回配置好的 `Progress` 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use workflow::base::indicator::Progress;
    ///
    /// let progress = Progress::new_unknown("Downloading...");
    /// // 执行操作
    /// progress.finish();
    /// ```
    pub fn new_unknown(message: impl AsRef<str>) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap(),
        );
        pb.set_message(message.as_ref().to_string());
        pb.enable_steady_tick(Duration::from_millis(100));

        Self { inner: pb }
    }

    /// 增加进度（按单位数）
    ///
    /// # 参数
    ///
    /// * `delta` - 增加的数量
    ///
    /// # 示例
    ///
    /// ```rust
    /// use workflow::base::indicator::Progress;
    ///
    /// let progress = Progress::new(100, "Processing...");
    /// progress.inc(1); // 增加 1
    /// ```
    pub fn inc(&self, delta: u64) {
        self.inner.inc(delta);
    }

    /// 增加进度（按字节数）
    ///
    /// # 参数
    ///
    /// * `delta` - 增加的字节数
    ///
    /// # 示例
    ///
    /// ```rust
    /// use workflow::base::indicator::Progress;
    ///
    /// let progress = Progress::new_download(1024 * 1024, "Downloading...");
    /// progress.inc_bytes(1024); // 增加 1024 字节
    /// ```
    pub fn inc_bytes(&self, delta: u64) {
        self.inner.inc(delta);
    }

    /// 设置当前位置
    ///
    /// # 参数
    ///
    /// * `pos` - 当前位置
    ///
    /// # 示例
    ///
    /// ```rust
    /// use workflow::base::indicator::Progress;
    ///
    /// let progress = Progress::new(100, "Processing...");
    /// progress.set_position(50); // 设置为 50%
    /// ```
    pub fn set_position(&self, pos: u64) {
        self.inner.set_position(pos);
    }

    /// 更新显示的消息
    ///
    /// # 参数
    ///
    /// * `message` - 新的消息文本
    ///
    /// # 示例
    ///
    /// ```rust
    /// use workflow::base::indicator::Progress;
    ///
    /// let progress = Progress::new(100, "Starting...");
    /// progress.update_message("Processing...");
    /// ```
    pub fn update_message(&self, message: impl AsRef<str>) {
        self.inner.set_message(message.as_ref().to_string());
    }

    /// 完成并清除进度条
    ///
    /// # 示例
    ///
    /// ```rust
    /// use workflow::base::indicator::Progress;
    ///
    /// let progress = Progress::new(100, "Processing...");
    /// progress.finish();
    /// ```
    pub fn finish(self) {
        self.inner.finish_and_clear();
    }

    /// 完成并清除进度条（不需要 move，用于 Mutex 中）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use workflow::base::indicator::Progress;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let progress = Arc::new(Mutex::new(Progress::new(100, "Processing...")));
    /// {
    ///     let pb = progress.lock().unwrap();
    ///     pb.finish_ref();
    /// }
    /// ```
    pub fn finish_ref(&self) {
        self.inner.finish_and_clear();
    }

    /// 完成进度条并显示完成消息
    ///
    /// # 参数
    ///
    /// * `message` - 完成消息
    ///
    /// # 示例
    ///
    /// ```rust
    /// use workflow::base::indicator::Progress;
    ///
    /// let progress = Progress::new(100, "Processing...");
    /// progress.finish_with_message("Completed!");
    /// ```
    pub fn finish_with_message(self, message: impl AsRef<str>) {
        self.inner.finish_with_message(message.as_ref().to_string());
    }
}
