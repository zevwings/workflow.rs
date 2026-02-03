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
