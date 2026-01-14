//! LLM 客户端模块
//!
//! 本模块提供了统一配置驱动的 LLM 客户端实现，支持 OpenAI、DeepSeek 和代理 API。
//!
//! ## 架构设计
//!
//! 本模块通过 `LLMConfigProvider` trait 实现依赖倒置，使 LLM 客户端独立于具体的配置实现。
//! 配置适配器在 `infra::adapters::config` 模块中实现，将 `Settings` 适配为配置提供者。

pub mod branch;
pub mod client;
pub mod pr;

// 重新导出核心 API（从 client 子模块）
#[allow(unused_imports)]
pub use client::LLMClient;
pub use client::LLMConfigProvider;
pub use client::LLMRequestParams;
pub use client::{
    find_language, get_language_instruction, get_language_requirement,
    get_supported_language_codes, get_supported_language_display_names, SupportedLanguage,
    SUPPORTED_LANGUAGES,
};

// 重新导出业务模块 API
pub use branch::TranslateLLM;
pub use pr::{
    CreateGenerator, FileSummaryGenerator, PullRequestContent, PullRequestReword,
    PullRequestSummary, RewordGenerator, SummaryGenerator,
};
