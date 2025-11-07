//! Git 操作模块
//! 包含 Git 命令封装、仓库类型检测等功能

mod branch;
mod commit;
mod pre_commit;
mod repo;
mod stash;
mod types;

// 重新导出所有公共 API
pub use commit::Git;
pub use types::RepoType;
