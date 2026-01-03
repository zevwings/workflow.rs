//! Git 客户端封装层
//!
//! 本模块提供了对 Git 操作的统一封装，使用 GitCommand 执行 git 命令，隐藏实现细节，提供统一的错误处理。
//!
//! ## 模块结构
//!
//! - `repository` - Git 仓库封装（`GitRepository`）
//! - `remote` - Git 远程仓库封装（`GitRemote`）

mod remote;
mod repository;

pub use remote::GitRemote;
pub use repository::GitRepository;
