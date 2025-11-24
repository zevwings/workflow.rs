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
}
