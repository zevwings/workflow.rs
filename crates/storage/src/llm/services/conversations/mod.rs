//! LLM 对话模块
//!
//! 提供业务逻辑层，负责 prompt 构建、参数验证和业务规则。

mod branch_name;
mod commit_message;
mod create;
mod file_summary;
mod pr_content;
mod reword;
mod summary;
mod translate;
mod verify;

// 导出给 services 模块使用
pub(crate) use branch_name::BranchNameConversation;
pub(crate) use commit_message::CommitMessageConversation;
pub(crate) use create::CreateConversation;
pub(crate) use file_summary::FileSummaryConversation;
pub(crate) use pr_content::PrContentConversation;
pub(crate) use reword::RewordConversation;
pub(crate) use summary::SummaryConversation;
pub(crate) use translate::TranslateConversation;
pub(crate) use verify::VerifyConversation;
