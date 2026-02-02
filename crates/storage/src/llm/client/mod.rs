//! LLM 客户端核心模块
//!
//! 本模块提供了统一配置驱动的 LLM 客户端实现，支持 OpenAI、DeepSeek 和代理 API。
//!
//! ## 架构设计
//!
//! 本模块通过 `LLMConfigContext` trait 实现依赖倒置，使 LLM 客户端独立于具体的配置实现。

pub mod context;
pub mod core;
pub(crate) mod response;

// 重新导出 API
pub use context::LLMConfigContextImpl;
pub use core::{LLMClient, LLMClientImpl, LLMRequestParameters};
