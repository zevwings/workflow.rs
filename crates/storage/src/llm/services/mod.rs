//! LLM 服务模块
//!
//! 提供 LLM 相关的服务实现，包括对话、服务、解析器和提示词。

mod conversation;
mod conversations;
mod parsers;
pub(crate) mod prompt;
mod service;

// 重新导出（仅限模块内部使用）
pub(crate) use conversation::LLMConversation;

// 重新导出
pub use service::{LLMService, LLMServiceImpl};
