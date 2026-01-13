//! 消息输出模块

use crate::base::interactive::error::Result;
use crate::base::interactive::style::{get_theme, Theme};
use color_eyre::eyre;
use std::io::Write;
use std::sync::{Mutex, OnceLock};

/// 消息输出器
pub struct Message {
    theme: Theme,
    writer: Box<dyn Write + Send>,
}

/// 全局消息输出器的便捷引用
///
/// 这个类型提供了对全局 `Message` 单例的便捷访问，自动处理锁的获取和释放。
/// 可以直接调用方法，无需手动处理 `lock().unwrap()`。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::interactive::Message;
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
    /// 输出信息
    pub fn info(&self, msg: impl AsRef<str>) -> Result<()> {
        Message::global_mutex().lock().unwrap().info(msg)
    }

    /// 输出成功信息
    pub fn success(&self, msg: impl AsRef<str>) -> Result<()> {
        Message::global_mutex().lock().unwrap().success(msg)
    }

    /// 输出警告信息
    pub fn warning(&self, msg: impl AsRef<str>) -> Result<()> {
        Message::global_mutex().lock().unwrap().warning(msg)
    }

    /// 输出错误信息
    pub fn error(&self, msg: impl AsRef<str>) -> Result<()> {
        Message::global_mutex().lock().unwrap().error(msg)
    }

    /// 输出空行
    pub fn break_line(&self) -> Result<()> {
        Message::global_mutex().lock().unwrap().break_line()
    }

    /// 输出分隔线
    pub fn separator(&self, char: char, length: usize) -> Result<()> {
        Message::global_mutex().lock().unwrap().separator(char, length)
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
    /// use workflow::base::interactive::Message;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let msg = Message::global();
    /// msg.info("这是一条信息")?;
    /// msg.success("操作成功")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn global() -> MessageRef {
        // 确保单例已初始化
        static MESSAGE: OnceLock<Mutex<Message>> = OnceLock::new();
        MESSAGE.get_or_init(|| Mutex::new(Message::new()));
        MessageRef
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
        writeln!(self.writer, "{}", styled).map_err(|e| eyre::eyre!("IO error: {}", e))?;
        Ok(())
    }

    /// 输出成功信息
    pub fn success(&mut self, msg: impl AsRef<str>) -> Result<()> {
        let styled = self
            .theme
            .success
            .apply(&format!("✓ {}", msg.as_ref()), self.theme.enable_color);
        writeln!(self.writer, "{}", styled).map_err(|e| eyre::eyre!("IO error: {}", e))?;
        Ok(())
    }

    /// 输出警告信息
    pub fn warning(&mut self, msg: impl AsRef<str>) -> Result<()> {
        let styled = self
            .theme
            .warning
            .apply(&format!("⚠ {}", msg.as_ref()), self.theme.enable_color);
        writeln!(self.writer, "{}", styled).map_err(|e| eyre::eyre!("IO error: {}", e))?;
        Ok(())
    }

    /// 输出错误信息
    pub fn error(&mut self, msg: impl AsRef<str>) -> Result<()> {
        let styled =
            self.theme.error.apply(&format!("✗ {}", msg.as_ref()), self.theme.enable_color);
        writeln!(self.writer, "{}", styled).map_err(|e| eyre::eyre!("IO error: {}", e))?;
        Ok(())
    }

    /// 输出空行
    pub fn break_line(&mut self) -> Result<()> {
        writeln!(self.writer).map_err(|e| eyre::eyre!("IO error: {}", e))?;
        Ok(())
    }

    /// 输出分隔线
    pub fn separator(&mut self, char: char, length: usize) -> Result<()> {
        let line: String = std::iter::repeat_n(char, length).collect();
        writeln!(self.writer, "{}", line).map_err(|e| eyre::eyre!("IO error: {}", e))?;
        Ok(())
    }
}

impl Default for Message {
    fn default() -> Self {
        Self::new()
    }
}
