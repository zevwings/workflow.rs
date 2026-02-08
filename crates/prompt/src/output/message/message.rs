//! Message 结构体定义和实现

use crate::error::{PromptError, Result};
use crate::style::theme::{get_theme, Theme};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

/// 消息输出器
pub struct Message {
    theme: Theme,
    writer: Box<dyn Write + Send>,
}

/// 用于测试的模拟写入器
#[cfg(any(test, feature = "testing"))]
pub struct MockWriter {
    buffer: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
}

#[cfg(any(test, feature = "testing"))]
impl MockWriter {
    /// 创建新的模拟写入器
    pub fn new() -> Self {
        Self {
            buffer: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// 将捕获的输出转换为字符串
    pub fn output(&self) -> String {
        let buffer = self.buffer.lock().unwrap();
        String::from_utf8_lossy(&buffer).to_string()
    }

    /// 清空缓冲区
    pub fn clear(&self) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.clear();
    }

    /// 克隆内部缓冲区以供共享
    pub fn clone_buffer(&self) -> std::sync::Arc<std::sync::Mutex<Vec<u8>>> {
        std::sync::Arc::clone(&self.buffer)
    }
}

#[cfg(any(test, feature = "testing"))]
impl Default for MockWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(any(test, feature = "testing"))]
impl Write for MockWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(any(test, feature = "testing"))]
impl Clone for MockWriter {
    fn clone(&self) -> Self {
        Self {
            buffer: std::sync::Arc::clone(&self.buffer),
        }
    }
}

impl Message {
    /// 获取全局 Message 单例的便捷引用
    ///
    /// 返回进程级别的 Message 单例的便捷引用。
    /// 单例会在首次调用时初始化，后续调用会复用同一个实例。
    ///
    /// # 返回
    ///
    /// 返回 `MessageRef`，可以直接调用方法，无需手动处理锁。
    ///
    /// # 优势
    ///
    /// - 减少资源消耗：避免重复创建消息输出器实例
    /// - 线程安全：可以在多线程环境中安全使用
    /// - 统一管理：所有消息输出使用同一个实例
    /// - 便捷使用：无需手动处理 `lock().unwrap()`
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
    pub fn global() -> super::message_ref::MessageRef {
        // 确保单例已初始化
        static MESSAGE: OnceLock<Mutex<Message>> = OnceLock::new();
        MESSAGE.get_or_init(|| Mutex::new(Message::new()));
        super::message_ref::MessageRef
    }

    /// 获取全局 Message 单例的原始 Mutex 引用（高级用法）
    ///
    /// 如果需要直接访问 `Mutex<Message>`，可以使用此方法。
    /// 大多数情况下，应该使用 `global()` 方法。
    ///
    /// # 返回
    ///
    /// 返回 `Mutex<Message>` 的静态引用。
    pub fn global_mutex() -> &'static Mutex<Self> {
        static MESSAGE: OnceLock<Mutex<Message>> = OnceLock::new();
        MESSAGE.get_or_init(|| Mutex::new(Message::new()))
    }

    /// 创建新的消息输出器（私有方法，仅内部使用）
    fn new() -> Self {
        Self {
            theme: get_theme(),
            writer: Box::new(std::io::stdout()) as Box<dyn Write + Send>,
        }
    }

    /// 创建带自定义 writer 的消息输出器（用于测试）
    #[cfg(any(test, feature = "testing"))]
    pub fn with_writer<W: Write + Send + 'static>(writer: W) -> Self {
        Self {
            theme: get_theme(),
            writer: Box::new(writer),
        }
    }

    /// 创建不带颜色的消息输出器（用于测试验证输出内容）
    #[cfg(any(test, feature = "testing"))]
    pub fn with_writer_no_color<W: Write + Send + 'static>(writer: W) -> Self {
        let mut theme = get_theme();
        theme.enable_color = false;
        Self {
            theme,
            writer: Box::new(writer),
        }
    }

    /// 输出信息
    pub fn info(&mut self, msg: impl AsRef<str>) -> Result<()> {
        let styled = self.theme.info.apply(&format!("ℹ {}", msg.as_ref()), self.theme.enable_color);
        writeln!(self.writer, "{}", styled).map_err(PromptError::Io)?;
        Ok(())
    }

    /// 输出成功信息
    pub fn success(&mut self, msg: impl AsRef<str>) -> Result<()> {
        let styled = self
            .theme
            .success
            .apply(&format!("✓ {}", msg.as_ref()), self.theme.enable_color);
        writeln!(self.writer, "{}", styled).map_err(PromptError::Io)?;
        Ok(())
    }

    /// 输出警告信息
    pub fn warning(&mut self, msg: impl AsRef<str>) -> Result<()> {
        let styled = self
            .theme
            .warning
            .apply(&format!("⚠ {}", msg.as_ref()), self.theme.enable_color);
        writeln!(self.writer, "{}", styled).map_err(PromptError::Io)?;
        Ok(())
    }

    /// 输出错误信息
    pub fn error(&mut self, msg: impl AsRef<str>) -> Result<()> {
        let styled =
            self.theme.error.apply(&format!("✗ {}", msg.as_ref()), self.theme.enable_color);
        writeln!(self.writer, "{}", styled).map_err(PromptError::Io)?;
        Ok(())
    }

    /// 输出调试信息
    pub fn debug(&mut self, msg: impl AsRef<str>) -> Result<()> {
        let styled =
            self.theme.debug.apply(&format!("⚙ {}", msg.as_ref()), self.theme.enable_color);
        writeln!(self.writer, "{}", styled).map_err(PromptError::Io)?;
        Ok(())
    }

    /// 输出纯文本（无 emoji 前缀）
    ///
    /// 直接输出文本，不添加任何前缀或 emoji，但会应用主题样式（如果有）。
    pub fn print(&mut self, msg: impl AsRef<str>) -> Result<()> {
        // 使用 info 样式但不添加 emoji 前缀
        let styled = self.theme.info.apply(msg.as_ref(), self.theme.enable_color);
        writeln!(self.writer, "{}", styled).map_err(PromptError::Io)?;
        Ok(())
    }

    /// 输出空行
    pub fn break_line(&mut self) -> Result<()> {
        writeln!(self.writer).map_err(PromptError::Io)?;
        Ok(())
    }

    /// 输出分隔线
    pub fn separator(&mut self, char: char, length: usize) -> Result<()> {
        let line: String = std::iter::repeat_n(char, length).collect();
        writeln!(self.writer, "{}", line).map_err(PromptError::Io)?;
        Ok(())
    }

    /// 输出带文本的分隔线
    ///
    /// 在分隔线中间插入文本，文本前后用分隔符字符填充。
    /// 文本前后会自动添加空格。
    ///
    /// # 参数
    ///
    /// * `char` - 分隔符字符
    /// * `length` - 总长度
    /// * `text` - 要插入的文本
    pub fn separator_with_text(
        &mut self,
        char: char,
        length: usize,
        text: impl AsRef<str>,
    ) -> Result<()> {
        let text_str = format!("  {} ", text.as_ref());
        let text_len = text_str.chars().count();

        // 如果文本长度大于等于总长度，直接输出文本
        if text_len >= length {
            writeln!(self.writer, "{}", text_str).map_err(PromptError::Io)?;
            return Ok(());
        }

        // 计算左右两侧需要填充的字符数
        let remaining = length - text_len;
        let left_padding = remaining / 2;
        let right_padding = remaining - left_padding;

        // 生成分隔线
        let left_sep: String = std::iter::repeat_n(char, left_padding).collect();
        let right_sep: String = std::iter::repeat_n(char, right_padding).collect();

        writeln!(self.writer, "{}{}{}", left_sep, text_str, right_sep).map_err(PromptError::Io)?;
        Ok(())
    }
}

impl Default for Message {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_info() {
        let mock = MockWriter::new();
        let mut msg = Message::with_writer_no_color(mock.clone());

        msg.info("Test info message").unwrap();

        let output = mock.output();
        assert!(output.contains("ℹ"));
        assert!(output.contains("Test info message"));
    }

    #[test]
    fn test_message_success() {
        let mock = MockWriter::new();
        let mut msg = Message::with_writer_no_color(mock.clone());

        msg.success("Operation completed").unwrap();

        let output = mock.output();
        assert!(output.contains("✓"));
        assert!(output.contains("Operation completed"));
    }

    #[test]
    fn test_message_warning() {
        let mock = MockWriter::new();
        let mut msg = Message::with_writer_no_color(mock.clone());

        msg.warning("This is a warning").unwrap();

        let output = mock.output();
        assert!(output.contains("⚠"));
        assert!(output.contains("This is a warning"));
    }

    #[test]
    fn test_message_error() {
        let mock = MockWriter::new();
        let mut msg = Message::with_writer_no_color(mock.clone());

        msg.error("An error occurred").unwrap();

        let output = mock.output();
        assert!(output.contains("✗"));
        assert!(output.contains("An error occurred"));
    }

    #[test]
    fn test_message_debug() {
        let mock = MockWriter::new();
        let mut msg = Message::with_writer_no_color(mock.clone());

        msg.debug("Debug info").unwrap();

        let output = mock.output();
        assert!(output.contains("⚙"));
        assert!(output.contains("Debug info"));
    }

    #[test]
    fn test_message_print() {
        let mock = MockWriter::new();
        let mut msg = Message::with_writer_no_color(mock.clone());

        msg.print("Plain text").unwrap();

        let output = mock.output();
        assert!(output.contains("Plain text"));
        // print 不应该有 emoji 前缀
        assert!(!output.contains("ℹ"));
        assert!(!output.contains("✓"));
    }

    #[test]
    fn test_message_break_line() {
        let mock = MockWriter::new();
        let mut msg = Message::with_writer_no_color(mock.clone());

        msg.break_line().unwrap();

        let output = mock.output();
        assert_eq!(output, "\n");
    }

    #[test]
    fn test_message_separator() {
        let mock = MockWriter::new();
        let mut msg = Message::with_writer_no_color(mock.clone());

        msg.separator('-', 10).unwrap();

        let output = mock.output();
        assert!(output.contains("----------"));
    }

    #[test]
    fn test_message_separator_with_text() {
        let mock = MockWriter::new();
        let mut msg = Message::with_writer_no_color(mock.clone());

        msg.separator_with_text('-', 20, "Title").unwrap();

        let output = mock.output();
        assert!(output.contains("Title"));
        assert!(output.contains("-"));
    }

    #[test]
    fn test_message_separator_with_long_text() {
        let mock = MockWriter::new();
        let mut msg = Message::with_writer_no_color(mock.clone());

        // 文本比长度长时，应该只输出文本
        msg.separator_with_text('-', 5, "Very long text").unwrap();

        let output = mock.output();
        assert!(output.contains("Very long text"));
    }

    #[test]
    fn test_message_multiple_outputs() {
        let mock = MockWriter::new();
        let mut msg = Message::with_writer_no_color(mock.clone());

        msg.info("First").unwrap();
        msg.success("Second").unwrap();
        msg.error("Third").unwrap();

        let output = mock.output();
        assert!(output.contains("First"));
        assert!(output.contains("Second"));
        assert!(output.contains("Third"));
    }

    #[test]
    fn test_mock_writer_clear() {
        let mock = MockWriter::new();
        let mut msg = Message::with_writer_no_color(mock.clone());

        msg.info("Test").unwrap();
        assert!(!mock.output().is_empty());

        mock.clear();
        assert!(mock.output().is_empty());
    }

    #[test]
    fn test_message_with_unicode() {
        let mock = MockWriter::new();
        let mut msg = Message::with_writer_no_color(mock.clone());

        msg.info("你好世界").unwrap();

        let output = mock.output();
        assert!(output.contains("你好世界"));
    }
}
