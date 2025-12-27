//! Git2 客户端封装层
//!
//! 本模块提供了对 git2 库的统一封装，隐藏实现细节，提供统一的错误处理。
//!
//! ## 模块结构
//!
//! - `repository` - Git 仓库封装（`GitRepository`）
//! - `remote` - Git 远程仓库封装（`GitRemote`）

mod remote;
mod repository;

pub use remote::GitRemote;
pub use repository::GitRepository;
