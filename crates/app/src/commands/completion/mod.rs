//! Completion 管理命令模块
//!
//! 提供 Shell Completion 的生成、检查和移除功能。

mod check;
mod generate;
mod remove;

pub use check::CompletionCheckCommand;
pub use generate::CompletionGenerateCommand;
pub use remove::CompletionRemoveCommand;
