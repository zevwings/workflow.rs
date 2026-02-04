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
        let mut guard = Message::global_mutex().lock().map_err(|_| PromptError::LockPoisoned)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试会向 stdout 输出内容
    // 由于使用全局状态，测试之间可能有依赖

    #[test]
    fn test_message_ref_info() {
        let msg = MessageRef;
        let result = msg.info("Test info message");
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_ref_success() {
        let msg = MessageRef;
        let result = msg.success("Test success message");
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_ref_warning() {
        let msg = MessageRef;
        let result = msg.warning("Test warning message");
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_ref_error() {
        let msg = MessageRef;
        let result = msg.error("Test error message");
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_ref_debug() {
        let msg = MessageRef;
        let result = msg.debug("Test debug message");
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_ref_print() {
        let msg = MessageRef;
        let result = msg.print("Test print message");
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_ref_break_line() {
        let msg = MessageRef;
        let result = msg.break_line();
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_ref_separator() {
        let msg = MessageRef;
        let result = msg.separator('-', 40);
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_ref_separator_with_text() {
        let msg = MessageRef;
        let result = msg.separator_with_text('=', 60, "Section");
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_ref_empty_message() {
        let msg = MessageRef;
        let result = msg.info("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_ref_unicode_message() {
        let msg = MessageRef;
        let result = msg.info("中文信息 🎉");
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_ref_long_message() {
        let msg = MessageRef;
        let long_message = "A".repeat(1000);
        let result = msg.info(&long_message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_ref_multiple_calls() {
        let msg = MessageRef;
        assert!(msg.info("First").is_ok());
        assert!(msg.success("Second").is_ok());
        assert!(msg.warning("Third").is_ok());
    }

    #[test]
    fn test_message_ref_separator_zero_length() {
        let msg = MessageRef;
        let result = msg.separator('-', 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_ref_separator_unicode_char() {
        let msg = MessageRef;
        let result = msg.separator('─', 20);
        assert!(result.is_ok());
    }
}
