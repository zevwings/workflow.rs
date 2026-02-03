//! 全局配置模块
//!
//! 从 workflow.toml 配置文件读取全局应用配置

pub mod config;
pub mod github;
pub mod jira;
pub mod llm;
pub mod log;
pub mod repository;
pub mod verification_service;

// Re-export public types
pub use config::GlobalConfig;
pub use github::{
    GitHubAccount, GitHubAccountInfo, GitHubSettings, GitHubVerificationResult,
    GitHubVerificationSummary,
};
pub use jira::{JiraConfigInfo, JiraSettings, JiraVerificationResult, JiraVerificationStatus};
pub use llm::{
    LLMConfig, LLMProviderSettings, LLMSettings, LLMVerificationResult, LLMVerificationStatus,
};
pub use log::{LogConfigInfo, LogSettings, LogVerificationResult};
pub use repository::GlobalConfigRepository;
pub use verification_service::VerificationService;
