//! Completion 管理命令模块
//!
//! 提供 Shell Completion 的生成、检查和移除功能。

mod check;
mod cli;
mod generate;
mod remove;

// 重新导出 CLI 定义
pub use cli::CompletionCommand;

// 重新导出命令实现
pub use check::CompletionCheckCommand;
pub use generate::CompletionGenerateCommand;
pub use remove::CompletionRemoveCommand;
