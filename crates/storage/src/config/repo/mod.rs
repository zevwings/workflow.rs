//! 仓库配置模块
//!
//! 提供仓库配置的仓储和存储服务实现。

pub mod repository;

// Re-export public types
pub use repository::RepoConfigRepositoryImpl;
