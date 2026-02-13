//! LLM 模块
//!
//! 提供统一配置驱动的 LLM 客户端、Conversation 抽象与解析器。
//! 通过 `LLMConfigContext` trait 实现依赖倒置，客户端独立于具体配置实现。

pub use crate::{ChatCompletionResponse, LLMError, LLMRequestParameters};

pub trait LLMClient: Send + Sync + 'static {
    fn call(&self, params: &LLMRequestParameters) -> Result<ChatCompletionResponse, LLMError>;
}
