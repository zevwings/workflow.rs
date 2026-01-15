//! 输入对话框模块
//!
//! 提供文本输入功能，支持密码模式、验证器、占位符等

mod builder;
mod editor;
#[doc(hidden)]
pub mod macros; // 宏定义（通过 #[macro_export] 在 crate 根级别导出）
mod prompt;
mod validator;

pub use builder::InputBuilder;
pub use validator::{validators, ValidationResult, Validator};
