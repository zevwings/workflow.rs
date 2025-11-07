//! LLM 客户端模块
//!
//! 本模块提供了不同 LLM 提供商的客户端实现：
//! - `openai` - OpenAI API 客户端
//! - `deepseek` - DeepSeek API 客户端
//! - `proxy` - 代理 API 客户端

pub mod deepseek;
pub mod openai;
pub mod proxy;
