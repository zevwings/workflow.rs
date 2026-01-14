//! 翻译 LLM 服务模块
//!
//! 本模块提供了使用 LLM 将非英文文本翻译为英文的功能。

mod translate;
#[path = "translate.system.rs"]
mod translate_system;

pub use translate::TranslateLLM;
