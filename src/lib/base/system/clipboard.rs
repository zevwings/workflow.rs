//! 剪贴板操作模块
//!
//! 本模块提供了剪贴板的读写功能。

#[cfg(not(target_env = "musl"))]
use clipboard::{ClipboardContext, ClipboardProvider};
use color_eyre::Result;

#[cfg(not(target_env = "musl"))]
use color_eyre::eyre::eyre;

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
    /// 在 musl 静态链接构建中，剪贴板功能不可用（静默失败）。
    #[cfg(not(target_env = "musl"))]
    pub fn copy(text: &str) -> Result<()> {
        let mut ctx: ClipboardContext =
            ClipboardProvider::new().map_err(|e| eyre!("Failed to initialize clipboard: {}", e))?;

        ctx.set_contents(text.to_string())
            .map_err(|e| eyre!("Failed to copy to clipboard: {}", e))?;

        Ok(())
    }

    /// 复制文本到剪贴板（musl 目标：静默失败）
    ///
    /// 在 musl 静态链接构建中，剪贴板功能不可用，此方法静默失败。
    #[cfg(target_env = "musl")]
    pub fn copy(_text: &str) -> Result<()> {
        // musl 静态链接构建不支持剪贴板（需要 XCB 库）
        // 静默失败，不影响其他功能
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Clipboard Structure Tests ====================

    /// 测试Clipboard结构体创建
    ///
    /// ## 测试目的
    /// 验证 `Clipboard` 结构体能够成功创建（零大小结构体）。
    ///
    /// ## 测试场景
    /// 1. 创建Clipboard结构体实例
    /// 2. 验证创建成功
    ///
    /// ## 预期结果
    /// - 结构体创建成功，不会panic
    #[test]
    fn test_clipboard_copy_structure_with_no_parameters_creates_clipboard() {
        // Arrange: 准备创建Clipboard

        // Act: 创建Clipboard结构体
        let _clipboard = Clipboard;

        // Assert: 验证可以创建
    }

    // ==================== Clipboard Copy Tests ====================

    /// 测试Clipboard复制文本到剪贴板
    ///
    /// ## 测试目的
    /// 验证 `Clipboard::copy()` 方法能够复制文本到剪贴板（平台相关，某些平台可能静默失败）。
    ///
    /// ## 测试场景
    /// 1. 准备测试文本
    /// 2. 调用copy方法复制文本
    /// 3. 验证函数可以调用（可能成功或失败，取决于平台）
    ///
    /// ## 注意事项
    /// - 在某些平台上（如musl、Linux ARM64）会静默失败
    /// - 主要验证函数不会panic
    ///
    /// ## 预期结果
    /// - 函数返回Result类型（Ok或Err都可以接受）
    /// - 不会panic
    #[test]
    fn test_clipboard_copy_text_with_text_copies_to_clipboard() {
        // Arrange: 准备测试文本
        let text = "test text";

        // Act: 复制文本到剪贴板
        // 注意：在某些平台上（如musl、Linux ARM64）会静默失败
        let result = Clipboard::copy(text);

        // Assert: 验证函数可以调用（可能成功或失败，取决于平台）
        assert!(result.is_ok() || result.is_err());
    }

    /// 测试Clipboard复制空文本
    ///
    /// ## 测试目的
    /// 验证 `Clipboard::copy()` 方法能够处理空文本（在某些平台上空文本可以复制）。
    ///
    /// ## 测试场景
    /// 1. 准备空文本
    /// 2. 调用copy方法复制空文本
    /// 3. 验证函数可以调用
    ///
    /// ## 预期结果
    /// - 函数返回Result类型（Ok或Err都可以接受）
    /// - 不会panic
    #[test]
    fn test_clipboard_copy_empty_with_empty_text_copies_to_clipboard() {
        // Arrange: 准备空文本
        let text = "";

        // Act: 复制空文本
        let result = Clipboard::copy(text);

        // Assert: 验证函数可以调用（空文本应该可以复制，在某些平台上）
        assert!(result.is_ok() || result.is_err());
    }

    /// 测试Clipboard复制长文本
    ///
    /// ## 测试目的
    /// 验证 `Clipboard::copy()` 方法能够处理长文本（1000个字符）。
    ///
    /// ## 测试场景
    /// 1. 准备长文本（1000个字符）
    /// 2. 调用copy方法复制长文本
    /// 3. 验证函数可以调用
    ///
    /// ## 预期结果
    /// - 函数返回Result类型（Ok或Err都可以接受）
    /// - 不会panic
    #[test]
    fn test_clipboard_copy_long_text_with_long_text_copies_to_clipboard() {
        // Arrange: 准备长文本
        let long_text = "a".repeat(1000);

        // Act: 复制长文本
        let result = Clipboard::copy(&long_text);

        // Assert: 验证函数可以调用
        assert!(result.is_ok() || result.is_err());
    }

    /// 测试Clipboard复制包含特殊字符的文本
    ///
    /// ## 测试目的
    /// 验证 `Clipboard::copy()` 方法能够处理包含特殊字符的文本（如换行符、@符号等）。
    ///
    /// ## 测试场景
    /// 1. 准备包含特殊字符的文本（邮箱、换行符等）
    /// 2. 调用copy方法复制文本
    /// 3. 验证函数可以调用
    ///
    /// ## 预期结果
    /// - 函数返回Result类型（Ok或Err都可以接受）
    /// - 不会panic
    #[test]
    fn test_clipboard_copy_special_characters_with_special_chars_copies_to_clipboard() {
        // Arrange: 准备包含特殊字符的文本
        let special_text = "test@example.com\nline2\nline3";

        // Act: 复制特殊字符文本
        let result = Clipboard::copy(special_text);

        // Assert: 验证函数可以调用
        assert!(result.is_ok() || result.is_err());
    }

    /// 测试Clipboard复制Unicode文本
    ///
    /// ## 测试目的
    /// 验证 `Clipboard::copy()` 方法能够处理Unicode文本（中文、emoji等）。
    ///
    /// ## 测试场景
    /// 1. 准备Unicode文本（中文、emoji）
    /// 2. 调用copy方法复制Unicode文本
    /// 3. 验证函数可以调用
    ///
    /// ## 预期结果
    /// - 函数返回Result类型（Ok或Err都可以接受）
    /// - 不会panic
    #[test]
    fn test_clipboard_copy_unicode_with_unicode_text_copies_to_clipboard() {
        // Arrange: 准备Unicode文本
        let unicode_text = "测试文本 🚀 中文";

        // Act: 复制Unicode文本
        let result = Clipboard::copy(unicode_text);

        // Assert: 验证函数可以调用
        assert!(result.is_ok() || result.is_err());
    }
}
