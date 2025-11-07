//! 剪贴板操作模块
//!
//! 本模块提供了剪贴板的读写功能。

use anyhow::Result;
use clipboard::{ClipboardContext, ClipboardProvider};

/// 剪贴板操作模块
///
/// 提供复制和读取剪贴板内容的功能。
pub struct Clipboard;

impl Clipboard {
    /// 复制文本到剪贴板
    ///
    /// 将指定的文本复制到系统剪贴板。
    ///
    /// # 参数
    ///
    /// * `text` - 要复制的文本
    ///
    /// # 错误
    ///
    /// 如果复制失败，返回相应的错误信息。
    pub fn copy(text: &str) -> Result<()> {
        let mut ctx: ClipboardContext = ClipboardProvider::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize clipboard: {}", e))?;

        ctx.set_contents(text.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to copy to clipboard: {}", e))?;

        Ok(())
    }

    /// 从剪贴板读取文本
    ///
    /// 从系统剪贴板读取文本内容。
    ///
    /// # 返回
    ///
    /// 返回剪贴板中的文本内容。
    ///
    /// # 错误
    ///
    /// 如果读取失败，返回相应的错误信息。
    #[allow(dead_code)]
    pub fn read() -> Result<String> {
        let mut ctx: ClipboardContext = ClipboardProvider::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize clipboard: {}", e))?;

        let contents = ctx
            .get_contents()
            .map_err(|e| anyhow::anyhow!("Failed to read from clipboard: {}", e))?;

        Ok(contents)
    }
}
