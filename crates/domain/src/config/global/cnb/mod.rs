//! CNB 配置业务域
//!
//! 包含 CNB 配置数据和验证结果类型

pub mod config;
pub mod verification;

// Re-export public types
pub use config::{CNBAccount, CNBSettings};
pub use verification::{CNBAccountInfo, CNBVerificationResult, CNBVerificationSummary};
