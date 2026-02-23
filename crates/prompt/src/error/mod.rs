//! Prompt 错误模块

#[allow(clippy::module_inception)]
mod error;

pub use error::{is_user_cancelled, PromptError, Result};
