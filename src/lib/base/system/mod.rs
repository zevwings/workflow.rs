//! 系统交互模块
//!
//! 提供平台检测、浏览器操作和剪贴板操作相关的工具函数。

pub mod browser;
pub mod clipboard;
pub mod platform;

// 重新导出公共 API
pub use browser::Browser;
pub use clipboard::Clipboard;
pub use platform::Platform;
