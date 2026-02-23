//! 验证服务接口
//!
//! 负责验证全局配置的有效性

use crate::config::error::ConfigError;
use crate::config::global::{
    github::verification::GitHubVerificationResult, jira::verification::JiraVerificationResult,
    llm::verification::LLMVerificationResult, log::verification::LogVerificationResult,
    ssh::verification::SshVerificationResult,
};

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

    /// 验证 SSH 配置（检查 ssh-agent 状态和已加载密钥）
    fn verify_ssh_config(&self) -> Result<SshVerificationResult, ConfigError>;
}
