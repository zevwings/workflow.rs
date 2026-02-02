//! 平台特定实现模块
//!
//! 包含各个平台的工作流阶段实现。

pub mod cnb;
pub mod github;
pub mod jira;
pub mod llm;
pub mod log;

pub use cnb::cnb_stage;
pub use github::github_stage;
pub use jira::jira_stage;
pub use llm::llm_stage;
pub use log::log_stage;
