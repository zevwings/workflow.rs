//! LLM 客户端模块
//!
//! 本模块提供了统一配置驱动的 LLM 客户端实现，支持 OpenAI、DeepSeek 和代理 API。

pub mod llm_client;
pub mod types;

// 重新导出 API
#[allow(unused_imports)]
pub use llm_client::LLMClient;
pub use types::LLMRequestParams;
