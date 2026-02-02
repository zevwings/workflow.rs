//! LLM 客户端模块
//!
//! 本模块提供了统一配置驱动的 LLM 客户端实现，支持 OpenAI、DeepSeek 和代理 API。
//!
//! ## 架构设计
//!
//! 本模块通过 `LLMConfigContext` trait 实现依赖倒置，使 LLM 客户端独立于具体的配置实现。

pub(crate) mod client;
pub(crate) mod repository;
pub(crate) mod services;

// 重新导出 client
pub use client::{LLMClient, LLMClientImpl, LLMConfigContextImpl};

// 重新导出 repository
pub use repository::LLMRepositoryImpl;

// 重新导出 services
pub use services::{LLMService, LLMServiceImpl};
