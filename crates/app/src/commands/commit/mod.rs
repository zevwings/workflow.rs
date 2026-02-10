//! 提交操作命令

#[cfg(feature = "develop")]
mod cli;
#[cfg(feature = "develop")]
pub mod create;
#[cfg(feature = "develop")]
pub mod diff;
#[cfg(feature = "develop")]
pub mod files;
#[cfg(feature = "develop")]
pub mod to_merge;

// 重新导出 CLI 定义
#[cfg(feature = "develop")]
pub use cli::{CommitCommand, CommitSubcommand};

// 重新导出命令实现
#[cfg(feature = "develop")]
pub use create::CommitCreateCommand;
#[cfg(feature = "develop")]
pub use diff::CommitDiffCommand;
#[cfg(feature = "develop")]
pub use files::CommitFilesCommand;
#[cfg(feature = "develop")]
pub use to_merge::CommitToMergeCommand;
