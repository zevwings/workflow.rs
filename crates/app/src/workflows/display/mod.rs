//! 验证结果展示模块
//!
//! 提供验证结果的格式化显示功能，将 domain 层的验证结果转换为表格和消息输出。

mod attachment;
mod cnb;
mod formatter;
mod github;
mod jira;
mod llm;
mod log;

pub use attachment::AttachmentRow;
pub use cnb::CNBAccountRow;
pub use formatter::VerificationResultFormatter;
pub use github::GitHubAccountRow;
pub use jira::JiraConfigRow;
pub use llm::LLMConfigRow;
