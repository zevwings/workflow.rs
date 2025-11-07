//! LLM（大语言模型）模块
//!
//! 本模块提供了使用 LLM 生成 Pull Request 内容的功能，包括：
//! - 根据 commit 标题生成分支名和 PR 标题
//! - 支持多种 LLM 提供商（OpenAI、DeepSeek、代理 API）
//!
//! ## 模块结构
//!
//! - `pr_llm` - PR 内容生成（`PullRequestLLM` 结构体）
//! - `client` - LLM 客户端实现（OpenAI、DeepSeek、代理）

mod client;
mod pr_llm;

pub use pr_llm::{PullRequestContent, PullRequestLLM};
