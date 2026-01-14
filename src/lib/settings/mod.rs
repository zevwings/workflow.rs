//! Settings 模块
//! 用于管理应用程序的各种设置和配置

pub mod github;
pub mod jira;
pub mod llm;
pub mod log;
pub mod paths;
#[allow(clippy::module_inception)]
pub mod settings;

// 导出公共类型和函数
pub use github::{
    GitHubAccount, GitHubAccountInfo, GitHubAccountListRow, GitHubAccountRow, GitHubSettings,
    GitHubVerificationResult, GitHubVerificationSummary,
};
pub use jira::{
    JiraConfigInfo, JiraConfigRow, JiraSettings, JiraVerificationResult, JiraVerificationStatus,
};
pub use llm::{
    LLMConfigInfo, LLMConfigRow, LLMProviderSettings, LLMSettings, LLMVerificationResult,
    LLMVerificationStatus,
};
pub use log::{default_download_base_dir, LogConfigInfo, LogSettings};
pub use paths::Paths;
pub use settings::Settings;
