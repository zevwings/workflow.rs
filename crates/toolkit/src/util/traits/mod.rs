//! Trait 扩展模块
//!
//! 本模块提供了各种扩展 trait，为现有类型添加便捷方法。
//!
//! ## 模块结构
//!
//! - `browser_ext` - 浏览器扩展 trait（`BrowserExt`）和浏览器类型（`Browser`）
//! - `clipboard_ext` - 剪贴板扩展 trait（`ClipboardExt`）
//! - `sensitive_ext` - 敏感字符串扩展 trait（`Sensitive`）
//! - `truncate_ext` - 字符串截断扩展 trait（`Truncate`）
//! - `path_ext` - 路径显示扩展 trait（`PathExt`）
//! - `size_ext` - 文件大小显示扩展 trait（`SizeExt`）

pub(crate) mod browser_ext;
pub(crate) mod clipboard_ext;
pub(crate) mod path_ext;
pub(crate) mod sensitive_ext;
pub(crate) mod size_ext;
pub(crate) mod truncate_ext;

// 重新导出所有扩展 trait 和类型
pub use browser_ext::{Browser, BrowserError, BrowserExt};
pub use clipboard_ext::{ClipboardError, ClipboardExt};
pub use path_ext::PathExt;
pub use sensitive_ext::Sensitive;
pub use size_ext::SizeExt;
pub use truncate_ext::Truncate;
