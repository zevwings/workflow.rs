//! LLM 模块
//!
//! 提供统一配置驱动的 LLM 客户端、Conversation 抽象与解析器。
//! 通过 `LLMConfigContext` trait 实现依赖倒置，客户端独立于具体配置实现。

#[macro_use]
pub(crate) mod logger;
pub(crate) mod bootstrap;
pub(crate) mod client;
pub(crate) mod context;
pub(crate) mod conversation;
pub(crate) mod error;
pub(crate) mod language;
pub(crate) mod parsers;
pub(crate) mod types;

pub use bootstrap::register_llm;
pub use context::{IntoLLMConfig, LLMConfigContext};
pub use conversation::{IntoLLMRequestParameters, LLMConversation};
pub use error::LLMError;
pub use language::SupportedLanguage;
pub use parsers::{JsonParseMode, JsonParser, TextParser};
pub use types::{
    ChatCompletionChoice, ChatCompletionResponse, ChatMessage, LLMRequestParameters, Usage,
};

#[cfg(any(test, feature = "testing"))]
pub mod testing;

pub trait LLMClient: Send + Sync + 'static {
    fn call(&self, params: &LLMRequestParameters) -> Result<String, LLMError>;
}
