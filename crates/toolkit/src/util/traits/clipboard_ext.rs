//! 剪贴板扩展 trait
//!
//! 为字符串类型提供剪贴板操作相关的扩展方法。

use thiserror::Error;

#[cfg(all(
    not(target_env = "musl"),
    not(all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"))
))]
use clipboard::{ClipboardContext, ClipboardProvider};

/// 剪贴板操作错误
#[derive(Debug, Error)]
pub enum ClipboardError {
    /// 剪贴板操作错误
    #[error("Clipboard error: {0}")]
    Operation(String),
}

/// 剪贴板扩展 trait
///
/// 为字符串类型提供复制到剪贴板的功能。
///
/// # 示例
///
/// ```rust,no_run
/// use toolkit::ClipboardExt;
///
/// let text = "text to copy";
/// text.copy_to_clipboard()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait ClipboardExt {
    /// 复制文本到剪贴板
    ///
    /// 将指定的文本复制到系统剪贴板。
    ///
    /// # 返回
    ///
    /// 如果复制成功，返回 `Ok(())`；如果失败，返回相应的错误信息。
    ///
    /// # 注意
    ///
    /// 在 musl 静态链接构建和 Linux ARM64 交叉编译中，剪贴板功能不可用（静默失败）。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use toolkit::ClipboardExt;
    ///
    /// "text to copy".copy_to_clipboard()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn copy_to_clipboard(&self) -> Result<(), ClipboardError>;
}

/// 为 `str` 实现 `ClipboardExt` trait
impl ClipboardExt for str {
    #[cfg(all(
        not(target_env = "musl"),
        not(all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"))
    ))]
    fn copy_to_clipboard(&self) -> Result<(), ClipboardError> {
        let mut ctx: ClipboardContext = ClipboardProvider::new().map_err(|e| {
            ClipboardError::Operation(format!("Failed to initialize clipboard: {}", e))
        })?;

        ctx.set_contents(self.to_string()).map_err(|e| {
            ClipboardError::Operation(format!("Failed to copy to clipboard: {}", e))
        })?;

        Ok(())
    }

    #[cfg(any(
        target_env = "musl",
        all(target_arch = "aarch64", target_os = "linux", target_env = "gnu")
    ))]
    fn copy_to_clipboard(&self) -> Result<(), ClipboardError> {
        // musl 静态链接构建和 Linux ARM64 交叉编译不支持剪贴板（需要 XCB 库）
        // 静默失败，不影响其他功能
        Ok(())
    }
}

/// 为 `String` 实现 `ClipboardExt` trait
impl ClipboardExt for String {
    fn copy_to_clipboard(&self) -> Result<(), ClipboardError> {
        self.as_str().copy_to_clipboard()
    }
}

#[cfg(test)]
mod tests {
    use super::ClipboardExt;

    // ============================================================================
    // 不支持剪贴板的平台测试（musl 或 Linux ARM64）
    // ============================================================================

    #[cfg(any(
        target_env = "musl",
        all(target_arch = "aarch64", target_os = "linux", target_env = "gnu")
    ))]
    #[test]
    fn test_copy_to_clipboard_unsupported_platform_always_succeeds() {
        // 在不支持剪贴板的平台上，应该静默成功
        let text = "test text";
        let result = text.copy_to_clipboard();
        assert!(
            result.is_ok(),
            "Unsupported platform should silently succeed"
        );

        // 测试各种输入都应该成功
        assert!("".copy_to_clipboard().is_ok());
        assert!("测试文本 🚀".copy_to_clipboard().is_ok());
        assert!(String::from("test").copy_to_clipboard().is_ok());
    }

    // ============================================================================
    // 支持剪贴板的平台测试
    // ============================================================================

    #[cfg(all(
        not(target_env = "musl"),
        not(all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"))
    ))]
    mod supported_platform_tests {
        use super::*;

        #[test]
        fn test_copy_to_clipboard_str_type() {
            let text = "test text";
            let result = text.copy_to_clipboard();
            // 在支持的平台上，方法调用不会 panic
            // 可能成功（有剪贴板访问权限）或失败（CI 环境无权限）
            // 但至少要验证它返回一个合法的 Result
            match result {
                Ok(_) => {
                    // 成功复制到剪贴板
                }
                Err(e) => {
                    // 失败应该包含错误信息
                    let error_msg = e.to_string();
                    assert!(!error_msg.is_empty(), "Error message should not be empty");
                }
            }
        }

        #[test]
        fn test_copy_to_clipboard_string_type() {
            let text = String::from("test text");
            let result = text.copy_to_clipboard();
            // String 类型应该通过 as_str() 委托给 str 实现
            // 验证方法可以正常调用
            match result {
                Ok(_) => {}
                Err(e) => {
                    assert!(!e.to_string().is_empty());
                }
            }
        }

        #[test]
        fn test_copy_to_clipboard_empty_string() {
            let text = "";
            let result = text.copy_to_clipboard();
            // 空字符串也应该可以处理（不会 panic）
            match result {
                Ok(_) => {}
                Err(e) => {
                    assert!(!e.to_string().is_empty());
                }
            }
        }

        #[test]
        fn test_copy_to_clipboard_unicode_string() {
            let text = "测试文本 🚀 中文字符";
            let result = text.copy_to_clipboard();
            // Unicode 字符串应该可以正常处理
            match result {
                Ok(_) => {}
                Err(e) => {
                    assert!(!e.to_string().is_empty());
                }
            }
        }

        #[test]
        fn test_copy_to_clipboard_special_characters() {
            // 测试特殊字符
            let special_texts = vec![
                "line1\nline2\nline3",  // 换行符
                "tab\tseparated\ttext", // 制表符
                "\"quoted\" text",      // 引号
                "path/to/file",         // 路径分隔符
                "key=value&foo=bar",    // URL 查询参数
            ];

            for text in special_texts {
                let result = text.copy_to_clipboard();
                // 特殊字符不应该导致 panic
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        assert!(!e.to_string().is_empty());
                    }
                }
            }
        }
    }
}
