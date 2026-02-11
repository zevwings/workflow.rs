//! 验证服务接口
//!
//! 负责验证全局配置的有效性

use super::{
    github::verification::GitHubVerificationResult, jira::verification::JiraVerificationResult,
    llm::verification::LLMVerificationResult, log::verification::LogVerificationResult,
};
use crate::config::error::ConfigError;

/// 验证服务接口
pub trait VerificationService: Send + Sync {
    /// 验证 Jira 配置
    fn verify_jira_config(&self) -> Result<JiraVerificationResult, ConfigError>;

    /// 验证 GitHub 配置
    fn verify_github_config(&self) -> Result<GitHubVerificationResult, ConfigError>;

    /// 验证 LLM 配置
    fn verify_llm_config(&self) -> Result<LLMVerificationResult, ConfigError>;

    /// 验证日志配置
    fn verify_log_config(&self) -> Result<LogVerificationResult, ConfigError>;
}
