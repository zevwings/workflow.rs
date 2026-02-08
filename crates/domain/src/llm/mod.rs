//! LLM 业务域
//!
//! 包含 LLM 相关的实体、仓储接口和错误类型

pub mod context;
pub mod entity;
pub mod error;
pub mod language;
pub mod repository;

pub use context::LLMConfigContext;
pub use entity::{PullRequestReword, PullRequestSummary};
pub use error::LLMError;
pub use language::SupportedLanguage;
pub use repository::LLMRepository;
