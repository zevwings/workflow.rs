//! 平台特定实现模块
//!
//! 包含各个平台的工作流阶段实现。

mod github;
mod jira;
mod llm;
mod log;
mod ssh;

pub use github::github_stage;
pub use jira::jira_stage;
pub use llm::llm_stage;
pub use log::log_stage;
pub use ssh::ssh_stage;
