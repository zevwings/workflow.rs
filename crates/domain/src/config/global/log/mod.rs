//! Log 配置业务域
//!
//! 包含 Log 配置数据和验证结果类型

pub mod config;
pub mod verification;

// Re-export public types
pub use config::LogSettings;
pub use verification::{LogConfigInfo, LogVerificationResult};
