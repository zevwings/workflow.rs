//! Utils 模块
//! 提供各种工具函数和实用工具

pub mod browser;
pub mod clipboard;
pub mod completion;
pub mod env;
pub mod logger;
pub mod proxy;
pub mod shell;
pub mod string;
pub mod uninstall;

// 重新导出 Logger
pub use logger::Logger;

// 重新导出 string 模块的函数，保持向后兼容
pub use string::{extract_jira_project, mask_sensitive_value, transform_to_branch_name};

// 重新导出 browser 和 clipboard
pub use browser::Browser;
pub use clipboard::Clipboard;

// 重新导出 env
pub use env::EnvFile;

// 重新导出 proxy
pub use proxy::{Proxy, ProxyInfo};

// 重新导出 completion
pub use completion::Completion;

// 重新导出 shell
pub use shell::{Shell, ShellInfo};

// 重新导出 uninstall
pub use uninstall::Uninstall;
