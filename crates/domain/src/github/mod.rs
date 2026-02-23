//! GitHub 业务域
//!
//! 包含 GitHub 相关的实体、仓储接口和错误类型

pub mod entity;
pub mod error;
pub mod repository;
pub mod verification;

pub use entity::GitHubUser;
pub use error::GitHubError;
pub use repository::GitHubRepository;
pub use verification::GitHubVerificationService;
