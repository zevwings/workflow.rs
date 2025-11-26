//! 剪贴板操作模块
//!
//! 本模块提供了剪贴板的读写功能。

use anyhow::Result;
#[cfg(all(
    not(target_env = "musl"),
    not(all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"))
))]
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
    ///
    /// # 注意
    ///
    /// 在 musl 静态链接构建和 Linux ARM64 交叉编译中，剪贴板功能不可用（静默失败）。
    #[cfg(all(
        not(target_env = "musl"),
        not(all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"))
    ))]
    pub fn copy(text: &str) -> Result<()> {
        let mut ctx: ClipboardContext = ClipboardProvider::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize clipboard: {}", e))?;

        ctx.set_contents(text.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to copy to clipboard: {}", e))?;

        Ok(())
    }

    /// 复制文本到剪贴板（musl 或 Linux ARM64 目标：静默失败）
    ///
    /// 在 musl 静态链接构建和 Linux ARM64 交叉编译中，剪贴板功能不可用，此方法静默失败。
    #[cfg(any(
        target_env = "musl",
        all(target_arch = "aarch64", target_os = "linux", target_env = "gnu")
    ))]
    pub fn copy(_text: &str) -> Result<()> {
        // musl 静态链接构建和 Linux ARM64 交叉编译不支持剪贴板（需要 XCB 库）
        // 静默失败，不影响其他功能
        Ok(())
    }
}
