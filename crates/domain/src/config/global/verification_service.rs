//! 验证服务接口
//!
//! 负责验证全局配置的有效性

use crate::errors::ServiceError;

use super::cnb::verification::CNBVerificationResult;
use super::github::verification::GitHubVerificationResult;
use super::jira::verification::JiraVerificationResult;
use super::llm::verification::LLMVerificationResult;
use super::log::verification::LogVerificationResult;

/// 验证服务接口
pub trait VerificationService: Send + Sync {
    /// 验证 Jira 配置
    fn verify_jira_config(&self) -> Result<JiraVerificationResult, ServiceError>;

    /// 验证 GitHub 配置
    fn verify_github_config(&self) -> Result<GitHubVerificationResult, ServiceError>;

    /// 验证 CNB 配置
    fn verify_cnb_config(&self) -> Result<CNBVerificationResult, ServiceError>;

    /// 验证 LLM 配置
    fn verify_llm_config(&self) -> Result<LLMVerificationResult, ServiceError>;

    /// 验证日志配置
    fn verify_log_config(&self) -> Result<LogVerificationResult, ServiceError>;
}
