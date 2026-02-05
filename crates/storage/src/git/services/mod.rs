//! Git 内部服务层
//!
//! 提供 git2 库的封装和业务逻辑实现。

pub mod blame;
pub mod branch;
pub mod commit;
pub mod context;
pub mod diff;
pub mod hooks;
pub mod merge;
pub mod remote;
pub mod stash;
pub mod tag;

pub use blame::{BlameService, BlameServiceImpl};
pub use branch::{BranchService, BranchServiceImpl};
pub use commit::{CommitService, CommitServiceImpl};
pub use context::{DiscoveredContext, GitContext, GitContextHolder};
pub use diff::{DiffService, DiffServiceImpl};
pub use hooks::{
    git_hooks, pre_commit_hooks, HookContext, HookResult, HookService, HookServiceImpl,
};
pub use merge::{MergeService, MergeServiceImpl};
pub use remote::{RemoteService, RemoteServiceImpl};
pub use stash::{StashService, StashServiceImpl};
pub use tag::{TagService, TagServiceImpl};
