//! 配置类型定义
//!
//! 包含应用程序的所有配置类型，这些是领域模型的一部分。

pub mod global;
pub mod repo;

// Re-export public types
pub use global::{
    CNBAccount,
    CNBAccountInfo,
    CNBSettings,
    CNBVerificationResult,
    CNBVerificationSummary,
    GitHubAccount,
    // Verification types
    GitHubAccountInfo,
    GitHubSettings,
    GitHubVerificationResult,
    GitHubVerificationSummary,
    GlobalConfig,
    GlobalConfigRepository,
    JiraConfigInfo,
    JiraSettings,
    JiraVerificationResult,
    JiraVerificationStatus,
    LLMConfig,
    LLMProviderSettings,
    LLMSettings,
    LLMVerificationResult,
    LLMVerificationStatus,
    LogConfigInfo,
    LogSettings,
    LogVerificationResult,
    VerificationService,
};
pub use repo::{
    BranchConfig, BranchTemplates, CommitTemplates, MCPConfig, MCPServerConfig, ProjectConfig,
    PullRequestsTemplates, RepoConfig, RepoConfigRepository, TemplateConfig, UserConfig,
};
