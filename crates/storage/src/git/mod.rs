//! Git2 存储层实现
//!
//! 基于 git2-rs 的 Git 操作具体实现，实现 domain 层定义的 Repository trait。
//!
//! # 架构设计
//!
//! - **GitContext**: 管理 git2::Repository 实例，提供仓库访问
//! - **GitRepositoryImpl**: 实现 domain::git::GitRepository trait
//!
//! # 使用示例
//!
//! ```rust,no_run
//! use storage::git::{GitContext, GitRepositoryImpl};
//! use domain::git::GitRepository;
//!
//! # fn main() -> anyhow::Result<()> {
//! // 创建上下文
//! let ctx = GitContext::discover()?;
//!
//! // 创建 Git 仓储（需要注入所有 services）
//! // let repo = GitRepositoryImpl::new(ctx, ...services...);
//!
//! // 获取当前分支
//! // let current = repo.get_current_branch()?;
//! // println!("当前分支: {}", current);
//! # Ok(())
//! # }
//! ```

pub mod services;

mod repository;

// 导出 testing 和 performance 模块供测试和基准测试使用
#[cfg(any(test, feature = "test-helpers"))]
pub mod testing;

#[cfg(any(test, feature = "test-helpers"))]
pub mod performance;

// Re-export public types
pub use repository::GitRepositoryImpl;
pub use services::{DiscoveredContext, GitContext, GitContextHolder};
