//! GitHub 业务域
//!
//! 包含 GitHub 相关的实体、仓储接口和错误类型

pub mod context;
pub mod entity;
pub mod error;
pub mod repository;

// Re-export public types
pub use context::GitHubContext;
pub use entity::GitHubUser;
pub use error::GitHubError;
pub use repository::GitHubRepository;
