//! 提交操作命令

mod cli;
pub mod create;
#[cfg(feature = "develop")]
pub mod diff;
#[cfg(feature = "develop")]
pub mod files;
#[cfg(feature = "develop")]
pub mod to_merge;

// 重新导出 CLI 定义
pub use cli::CommitCommand;
#[cfg(feature = "develop")]
pub use cli::CommitSubcommand;
// 重新导出命令实现
pub use create::CommitCreateCommand;
#[cfg(feature = "develop")]
pub use diff::CommitDiffCommand;
#[cfg(feature = "develop")]
pub use files::CommitFilesCommand;
#[cfg(feature = "develop")]
pub use to_merge::CommitToMergeCommand;
