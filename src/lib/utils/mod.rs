//! Utils 模块
//!
//! 本模块提供了各种工具函数和实用工具，包括：
//! - 代理环境变量管理（仅用于代理功能，读取、写入、删除）
//! - Shell 检测和管理（检测 shell 类型、配置路径）
//! - Completion 管理（配置、删除）
//! - 代理检测和管理（系统代理、环境变量代理）
//! - 日志输出（带颜色的日志宏）
//! - 字符串处理（敏感值隐藏）
//! - 浏览器和剪贴板操作
//! - 卸载工具
//!
//! ## 模块结构
//!
//! - `env` - 代理环境变量管理（`EnvFile`，仅用于代理功能）
//! - `shell` - Shell 检测和管理（`Shell`、`ShellInfo`）
//! - `completion` - Completion 管理（`Completion`）
//! - `proxy` - 代理检测和管理（`Proxy`、`ProxyInfo`）
//! - `logger` - 日志输出（`Logger`）
//! - `string` - 字符串处理工具
//! - `browser` - 浏览器操作（`Browser`）
//! - `clipboard` - 剪贴板操作（`Clipboard`）
//! - `uninstall` - 卸载工具（`Uninstall`）

pub mod browser;
pub mod clipboard;
pub mod completion;
pub mod confirm;
pub mod env;
pub mod logger;
pub mod proxy;
pub mod shell;
pub mod string;
pub mod uninstall;

// 重新导出 Logger 和 LogLevel
pub use logger::{LogLevel, Logger};

// 重新导出 string 模块的函数，保持向后兼容
pub use string::mask_sensitive_value;

// 重新导出 browser 和 clipboard
pub use browser::Browser;
pub use clipboard::Clipboard;

// 重新导出 env
pub use env::EnvFile;

// 重新导出 proxy
pub use proxy::{Proxy, ProxyDisableResult, ProxyEnableResult, ProxyInfo};

// 重新导出 completion
pub use completion::Completion;

// 重新导出 shell
pub use shell::{Shell, ShellInfo};

// 重新导出 uninstall
pub use uninstall::Uninstall;

// 重新导出 confirm
pub use confirm::confirm;
