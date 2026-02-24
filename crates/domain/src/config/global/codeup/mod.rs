//! Codeup 配置模块

pub mod config;
pub mod verification;

pub use config::CodeupSettings;
pub use verification::{CodeupConfigInfo, CodeupVerificationResult, CodeupVerificationStatus};
