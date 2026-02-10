//! 应用层库入口
//!
//! 暴露命令实现等公共 API，供各个二进制入口复用。

pub mod bootstrap;
pub mod commands;
/// CLI 定义与参数（`app::commands::cli` 的便捷重导出）
pub use commands::cli;
pub(crate) mod interactive;
pub(crate) mod logger;
