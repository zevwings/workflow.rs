//! 验证结果展示模块
//!
//! 提供验证结果的格式化显示功能，将 domain 层的验证结果转换为表格和消息输出。

mod attachment;
mod formatter;
mod github;
mod jira;
mod llm;
mod log;
mod ssh;

pub use formatter::VerificationResultFormatter;
