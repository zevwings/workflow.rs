//! 验证模块
//!
//! 提供环境检查和配置验证功能，用于 `check` 和 `setup` 命令。

mod config;
mod environment;

pub use config::ConfigVerifier;
pub use environment::EnvironmentVerifier;

// 重新导出验证结果类型（从 settings 模块）
pub use crate::base::settings::settings::{
    GitHubVerificationResult, JiraVerificationResult, LLMConfigInfo, LogConfigInfo,
    VerificationResult,
};
