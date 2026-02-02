//! MessageRef 结构体定义和实现

use super::message::Message;
use crate::error::{PromptError, Result};

/// 全局消息输出器的便捷引用
///
/// 这个类型提供了对全局 `Message` 单例的便捷访问，自动处理锁的获取和释放。
/// 可以直接调用方法，无需手动处理 `lock().unwrap()`。
///
/// # 示例
///
/// ```rust,no_run
/// use prompt::Message;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let msg = Message::global();
/// msg.info("这是一条信息")?;
/// msg.success("操作成功")?;
/// # Ok(())
/// # }
/// ```
pub struct MessageRef;

impl MessageRef {
    /// 辅助函数：获取锁并处理错误
    fn with_lock<F, T>(f: F) -> Result<T>
    where
        F: FnOnce(&mut Message) -> Result<T>,
    {
        let mut guard = Message::global_mutex()
            .lock()
            .map_err(|_| PromptError::LockPoisoned)?;
        f(&mut guard)
    }

    /// 输出信息
    pub fn info(&self, msg: impl AsRef<str>) -> Result<()> {
        Self::with_lock(|m| m.info(msg))
    }

    /// 输出成功信息
    pub fn success(&self, msg: impl AsRef<str>) -> Result<()> {
        Self::with_lock(|m| m.success(msg))
    }

    /// 输出警告信息
    pub fn warning(&self, msg: impl AsRef<str>) -> Result<()> {
        Self::with_lock(|m| m.warning(msg))
    }

    /// 输出错误信息
    pub fn error(&self, msg: impl AsRef<str>) -> Result<()> {
        Self::with_lock(|m| m.error(msg))
    }

    /// 输出调试信息
    pub fn debug(&self, msg: impl AsRef<str>) -> Result<()> {
        Self::with_lock(|m| m.debug(msg))
    }

    /// 输出纯文本（无 emoji 前缀）
    pub fn print(&self, msg: impl AsRef<str>) -> Result<()> {
        Self::with_lock(|m| m.print(msg))
    }

    /// 输出空行
    pub fn break_line(&self) -> Result<()> {
        Self::with_lock(|m| m.break_line())
    }

    /// 输出分隔线
    pub fn separator(&self, char: char, length: usize) -> Result<()> {
        Self::with_lock(|m| m.separator(char, length))
    }

    /// 输出带文本的分隔线
    pub fn separator_with_text(
        &self,
        char: char,
        length: usize,
        text: impl AsRef<str>,
    ) -> Result<()> {
        Self::with_lock(|m| m.separator_with_text(char, length, text))
    }
}
