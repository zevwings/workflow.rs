//! LLM 模块
//!
//! 提供统一配置驱动的 LLM 客户端、Conversation 抽象与解析器。
//! 通过 `LLMConfigContext` trait 实现依赖倒置，客户端独立于具体配置实现。

pub mod client;
pub mod context;
pub mod conversation;
pub mod error;
pub mod executor;
pub mod language;
pub mod parsers;
pub mod registry;
pub(crate) mod response;

pub use client::{LLMClient, LLMRequestParameters};
pub use context::LLMConfigContext;
pub use conversation::LLMConversation;
pub use error::LLMError;
pub use executor::LLMExecutor;
pub use language::SupportedLanguage;
pub use parsers::{JsonParser, TextParser};
pub use registry::register_llm;
