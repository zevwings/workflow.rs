//! 配置模块
//!
//! 提供配置适配器，实现各种配置提供者 trait。

pub mod global;
pub mod repo;

pub use global::{GlobalConfigRepositoryImpl, VerificationServiceImpl};
pub use repo::RepoConfigRepositoryImpl;
