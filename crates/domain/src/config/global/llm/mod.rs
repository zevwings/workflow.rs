//! LLM 配置业务域
//!
//! 包含 LLM 配置数据和验证结果类型

pub mod config;
pub mod verification;

// Re-export public types
pub use config::{LLMProviderSettings, LLMSettings};
pub use verification::{LLMConfig, LLMVerificationResult, LLMVerificationStatus};
