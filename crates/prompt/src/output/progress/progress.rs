//! Progress 结构体（兼容原 indicator::Progress API）
//!
//! 提供与原有 `indicator::Progress` 相同的 API，已完全替代原实现。

use crate::output::progress::bar::ProgressBar;
use crate::output::progress::builder::ProgressBarBuilder;

/// Progress 结构体（兼容原 indicator::Progress API）
///
/// 提供与原有 `indicator::Progress` 相同的 API，已完全替代原实现。
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
    pub fn new(total: u64, message: impl AsRef<str>) -> Self {
        Self {
            inner: ProgressBarBuilder::new(message.as_ref())
                .with_total(total)
                .start(),
        }
    }

    /// 创建一个新的进度条（用于下载，显示字节数）
    ///
    /// # 参数
    ///
    /// * `total_bytes` - 总字节数
    /// * `message` - 要显示的消息文本
    pub fn new_download(total_bytes: u64, message: impl AsRef<str>) -> Self {
        Self {
            inner: ProgressBarBuilder::new(message.as_ref())
                .with_total(total_bytes)
                .with_download_mode()
                .start(),
        }
    }

    /// 创建一个新的进度条（未知总数，使用 spinner 模式）
    ///
    /// # 参数
    ///
    /// * `message` - 要显示的消息文本
    pub fn new_unknown(message: impl AsRef<str>) -> Self {
        Self {
            inner: ProgressBarBuilder::new(message.as_ref()).start(),
        }
    }

    /// 增加进度（按单位数）
    pub fn inc(&self, delta: u64) {
        self.inner.inc(delta);
    }

    /// 增加进度（按字节数）
    pub fn inc_bytes(&self, delta: u64) {
        self.inner.inc_bytes(delta);
    }

    /// 设置当前位置
    pub fn set_position(&self, pos: u64) {
        self.inner.set_position(pos);
    }

    /// 更新显示的消息
    pub fn update_message(&self, message: impl AsRef<str>) {
        self.inner.update_message(message.as_ref());
    }

    /// 完成并清除进度条
    pub fn finish(self) {
        self.inner.stop();
    }

    /// 完成并清除进度条（不需要 move，用于 Mutex 中）
    pub fn finish_ref(&self) {
        self.inner.finish_ref();
    }

    /// 完成进度条并显示完成消息
    pub fn finish_with_message(self, message: impl AsRef<str>) {
        self.inner.finish_with_message(message.as_ref());
    }
}
