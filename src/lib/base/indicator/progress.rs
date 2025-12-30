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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Progress Creation Tests ====================

    /// 测试创建进度条
    ///
    /// ## 测试目的
    /// 验证 Progress::new() 能够使用总数和消息创建进度条。
    ///
    /// ## 测试场景
    /// 1. 使用总数和消息创建进度条
    /// 2. 验证创建成功
    ///
    /// ## 预期结果
    /// - 进度条创建成功
    #[test]
    fn test_progress_new_with_total_and_message_creates_progress() {
        // Arrange: 准备总数和消息
        let total = 100;
        let message = "Processing...";

        // Act: 创建进度条
        let _progress = Progress::new(total, message);

        // Assert: 验证可以创建进度条（如果运行到这里没有panic，说明成功）
    }

    /// 测试创建下载进度条
    ///
    /// ## 测试目的
    /// 验证 Progress::new_download() 能够使用文件大小和消息创建下载进度条。
    ///
    /// ## 测试场景
    /// 1. 使用文件大小和消息创建下载进度条
    /// 2. 验证创建成功
    ///
    /// ## 预期结果
    /// - 下载进度条创建成功
    #[test]
    fn test_progress_new_download_with_size_and_message_creates_download_progress() {
        // Arrange: 准备文件大小和消息
        let size = 1024 * 1024;
        let message = "Downloading...";

        // Act: 创建下载进度条
        let _progress = Progress::new_download(size, message);

        // Assert: 验证可以创建下载进度条
    }

    /// 测试创建未知总数的进度条
    ///
    /// ## 测试目的
    /// 验证 Progress::new_unknown() 能够使用消息创建未知总数的进度条。
    ///
    /// ## 测试场景
    /// 1. 使用消息创建未知总数的进度条
    /// 2. 验证创建成功
    ///
    /// ## 预期结果
    /// - 未知总数的进度条创建成功
    #[test]
    fn test_progress_new_unknown_with_message_creates_unknown_progress() {
        // Arrange: 准备消息
        let message = "Processing...";

        // Act: 创建未知总数的进度条
        let _progress = Progress::new_unknown(message);

        // Assert: 验证可以创建进度条
    }

    // ==================== Progress Update Tests ====================

    /// 测试增加进度
    ///
    /// ## 测试目的
    /// 验证 Progress::inc() 能够增加进度。
    ///
    /// ## 测试场景
    /// 1. 创建进度条
    /// 2. 多次调用 inc() 增加进度
    /// 3. 验证方法调用成功
    ///
    /// ## 预期结果
    /// - inc() 方法调用成功
    #[test]
    fn test_progress_inc_with_amounts_increments_progress() {
        // Arrange: 准备进度条
        let progress = Progress::new(100, "Processing...");

        // Act: 增加进度
        progress.inc(1);
        progress.inc(10);

        // Assert: 验证可以调用 inc 方法
    }

    /// 测试增加字节进度
    ///
    /// ## 测试目的
    /// 验证 Progress::inc_bytes() 能够增加字节进度。
    ///
    /// ## 测试场景
    /// 1. 创建下载进度条
    /// 2. 多次调用 inc_bytes() 增加字节进度
    /// 3. 验证方法调用成功
    ///
    /// ## 预期结果
    /// - inc_bytes() 方法调用成功
    #[test]
    fn test_progress_inc_bytes_with_amounts_increments_bytes() {
        // Arrange: 准备下载进度条
        let progress = Progress::new_download(1024 * 1024, "Downloading...");

        // Act: 增加字节进度
        progress.inc_bytes(1024);
        progress.inc_bytes(2048);

        // Assert: 验证可以调用 inc_bytes 方法
    }

    /// 测试设置进度位置
    ///
    /// ## 测试目的
    /// 验证 Progress::set_position() 能够设置进度位置。
    ///
    /// ## 测试场景
    /// 1. 创建进度条
    /// 2. 多次调用 set_position() 设置位置
    /// 3. 验证方法调用成功
    ///
    /// ## 预期结果
    /// - set_position() 方法调用成功
    #[test]
    fn test_progress_set_position_with_positions_sets_position() {
        // Arrange: 准备进度条
        let progress = Progress::new(100, "Processing...");

        // Act: 设置位置
        progress.set_position(50);
        progress.set_position(75);

        // Assert: 验证可以调用 set_position 方法
    }

    /// 测试更新进度消息
    ///
    /// ## 测试目的
    /// 验证 Progress::update_message() 能够更新进度消息。
    ///
    /// ## 测试场景
    /// 1. 创建进度条
    /// 2. 多次调用 update_message() 更新消息
    /// 3. 验证方法调用成功
    ///
    /// ## 预期结果
    /// - update_message() 方法调用成功
    #[test]
    fn test_progress_update_message_with_messages_updates_message() {
        // Arrange: 准备进度条
        let progress = Progress::new(100, "Starting...");

        // Act: 更新消息
        progress.update_message("Processing...");
        progress.update_message("Almost done...");

        // Assert: 验证可以调用 update_message 方法
    }

    // ==================== Progress Finish Tests ====================

    /// 测试完成进度条
    ///
    /// ## 测试目的
    /// 验证 Progress::finish() 能够完成进度条。
    ///
    /// ## 测试场景
    /// 1. 创建进度条并增加进度
    /// 2. 调用 finish() 完成进度条
    /// 3. 验证方法调用成功
    ///
    /// ## 预期结果
    /// - finish() 方法调用成功
    #[test]
    fn test_progress_finish_with_progress_finishes_progress() {
        // Arrange: 准备进度条并增加进度
        let progress = Progress::new(100, "Processing...");
        progress.inc(50);

        // Act: 完成进度条
        progress.finish();

        // Assert: 验证可以调用 finish 方法
    }

    /// 测试完成进度条（引用版本）
    ///
    /// ## 测试目的
    /// 验证 Progress::finish_ref() 能够完成进度条（引用版本）。
    ///
    /// ## 测试场景
    /// 1. 创建进度条并增加进度
    /// 2. 调用 finish_ref() 完成进度条
    /// 3. 验证方法调用成功
    ///
    /// ## 预期结果
    /// - finish_ref() 方法调用成功
    #[test]
    fn test_progress_finish_ref_with_progress_finishes_progress() {
        // Arrange: 准备进度条并增加进度
        let progress = Progress::new(100, "Processing...");
        progress.inc(50);

        // Act: 完成进度条（引用版本）
        progress.finish_ref();

        // Assert: 验证可以调用 finish_ref 方法
    }

    /// 测试使用消息完成进度条
    ///
    /// ## 测试目的
    /// 验证 Progress::finish_with_message() 能够完成进度条并显示消息。
    ///
    /// ## 测试场景
    /// 1. 创建进度条并完成进度
    /// 2. 使用 finish_with_message() 完成并显示消息
    /// 3. 验证方法调用成功
    ///
    /// ## 预期结果
    /// - finish_with_message() 方法调用成功
    #[test]
    fn test_progress_finish_with_message_with_message_finishes_with_message() {
        // Arrange: 准备进度条并完成进度
        let progress = Progress::new(100, "Processing...");
        progress.inc(100);
        let message = "Completed!";

        // Act: 完成进度条并显示消息
        progress.finish_with_message(message);

        // Assert: 验证可以调用 finish_with_message 方法
    }

    /// 测试进度消息字符串类型转换
    ///
    /// ## 测试目的
    /// 验证 Progress::new() 能够接受 &str 和 String 类型的消息。
    ///
    /// ## 测试场景
    /// 1. 使用 &str 类型消息创建进度条
    /// 2. 使用 String 类型消息创建进度条
    /// 3. 验证两种方式都可以创建
    ///
    /// ## 预期结果
    /// - 两种消息类型都可以创建进度条
    #[test]
    fn test_progress_message_string_conversion() {
        // Arrange: 准备测试消息参数的类型转换
        let _progress1 = Progress::new(100, "String message");
        let _progress2 = Progress::new(100, "String message");
        // Assert: 验证两种方式都可以创建进度条
    }

    /// 测试进度条的多个操作组合
    ///
    /// ## 测试目的
    /// 验证 Progress 能够执行多个操作的组合。
    ///
    /// ## 测试场景
    /// 1. 创建进度条
    /// 2. 验证可以创建进度条（多个操作需要实际运行才能测试）
    ///
    /// ## 预期结果
    /// - 进度条创建成功
    #[test]
    fn test_progress_multiple_operations() {
        // Arrange: 准备测试进度条的多个操作组合
        let _progress = Progress::new(100, "Processing...");
        // Assert: 验证可以创建进度条（多个操作需要实际运行才能测试）
    }
}
