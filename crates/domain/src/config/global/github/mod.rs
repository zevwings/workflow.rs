//! GitHub 配置业务域
//!
//! 包含 GitHub 配置数据和验证结果类型

pub mod config;
pub mod verification;

// Re-export public types
pub use config::{GitHubAccount, GitHubSettings};
pub use verification::{GitHubAccountInfo, GitHubVerificationResult, GitHubVerificationSummary};
