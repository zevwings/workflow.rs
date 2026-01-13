//! Fallback 机制模块
//!
//! 提供类型安全的 Fallback 处理器，用于在非交互式环境下自动降级

mod handler;
mod options;

pub use handler::{ExecuteFallback, FallbackHandler};
pub use options::FallbackOptions;
