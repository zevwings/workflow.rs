use anyhow::Result;
use clipboard::{ClipboardContext, ClipboardProvider};

/// 剪贴板操作模块
pub struct Clipboard;

impl Clipboard {
    /// 复制文本到剪贴板
    pub fn copy(text: &str) -> Result<()> {
        let mut ctx: ClipboardContext = ClipboardProvider::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize clipboard: {}", e))?;

        ctx.set_contents(text.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to copy to clipboard: {}", e))?;

        Ok(())
    }

    /// 从剪贴板读取文本
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
