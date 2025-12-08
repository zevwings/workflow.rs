//! Git 操作模块
//!
//! 本模块提供了 Git 仓库操作的完整功能，包括：
//! - 提交管理（状态检查、暂存、提交、推送）
//! - 分支管理（创建、切换、检查、获取默认分支）
//! - Cherry-pick 操作（应用提交、继续、中止、状态检查）
//! - 仓库检测（Git 仓库检测、远程仓库类型识别）
//! - 暂存管理（stash push/pop、冲突检测）
//! - Pre-commit hooks 支持（检测和执行）
//! - 配置管理（设置和读取 Git 全局配置）
//!
//! ## 模块结构
//!
//! - `commit` - Git 提交相关操作（`GitCommit` 结构体）
//! - `branch` - 分支管理操作
//! - `cherry_pick` - Cherry-pick 操作（`GitCherryPick` 结构体）
//! - `repo` - 仓库检测和类型识别
//! - `stash` - 暂存管理
//! - `pre_commit` - Pre-commit hooks 支持
//! - `config` - Git 配置管理（`GitConfig` 结构体）
//! - `types` - 类型定义（`RepoType` 枚举）

mod branch;
mod cherry_pick;
mod commit;
mod config;
mod helpers;
mod pre_commit;
mod repo;
mod stash;
mod types;

// 重新导出所有公共 API
pub use branch::{GitBranch, MergeStrategy};
pub use cherry_pick::GitCherryPick;
pub use commit::GitCommit;
pub use config::GitConfig;
pub use pre_commit::GitPreCommit;
pub use repo::GitRepo;
pub use stash::{GitStash, StashPopResult};
pub use types::RepoType;
