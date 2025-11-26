//! LLM 客户端模块
//!
//! 本模块提供了统一配置驱动的 LLM 客户端实现，支持 OpenAI、DeepSeek 和代理 API。

pub mod client;
pub mod languages;
pub mod types;

// 重新导出 API
#[allow(unused_imports)]
pub use client::LLMClient;
pub use languages::{
    find_language, get_language_instruction, get_language_requirement,
    get_supported_language_codes, get_supported_language_display_names, SupportedLanguage,
    SUPPORTED_LANGUAGES,
};
pub use types::LLMRequestParams;
