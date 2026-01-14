//! 配置初始化设置模块
//!
//! 交互式配置应用，保存到 TOML 配置文件（~/.workflow/config/workflow.toml）

mod command;
mod github;
mod jira;
mod llm;
mod log;
mod types;

pub use command::SetupCommand;
pub use types::{CollectedConfig, JiraConfig, LLMConfig, LogConfig};
