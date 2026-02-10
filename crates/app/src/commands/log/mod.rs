//! 日志操作命令

pub mod check;
mod cli;
pub mod setup;

// 重新导出 CLI 定义
pub use cli::LogCommand;

// 重新导出命令实现
pub use check::LogCheckCommand;
pub use setup::LogSetupCommand;
