//! 通用错误消息常量
//!
//! 统一管理项目中使用的错误消息，确保错误信息的一致性和用户体验。

pub mod check;
pub mod client;
pub mod file;
pub mod generator_creation;
pub mod validation_error;

// 向后兼容的模块别名
pub mod file_operations {
    pub use super::file::*;
}
