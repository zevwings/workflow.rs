//! Git 命令包装层
//!
//! 本模块提供了 Git 命令的统一封装，包括：
//! - `command` - 基础命令执行层和错误类型（GitCommand, GitError）
//! - `branch` - 分支操作命令（GitBranchCommand）
//! - `commit` - 提交操作命令（GitCommitCommand）
//! - `stash` - Stash 操作命令（GitStashCommand）
//! - `cherry_pick` - Cherry-pick 操作命令（GitCherryPickCommand）
//! - `tag` - Tag 操作命令（GitTagCommand）
//! - `repo` - 仓库操作命令（GitRepoCommand）
//! - `config` - 配置操作命令（GitConfigCommand）

// 基础模块
pub mod command;

// 功能命令模块
pub mod branch;
pub mod cherry_pick;
pub mod commit;
pub mod config;
pub mod repo;
pub mod stash;
pub mod tag;

// 导出基础类型
pub use command::{GitCommand, GitError};

// 导出功能命令
pub use branch::GitBranchCommand;
pub use cherry_pick::GitCherryPickCommand;
pub use commit::GitCommitCommand;
pub use config::GitConfigCommand;
pub use repo::GitRepoCommand;
pub use stash::{GitStashCommand, StashEntry};
pub use tag::GitTagCommand;
