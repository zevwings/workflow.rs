//! Codeup 业务域
//!
//! 包含 Codeup 相关的实体、仓储接口和错误类型

pub mod entity;
pub mod error;
pub mod repository;

pub use crate::config::CodeupSettings;
pub use entity::CodeupUser;
pub use error::CodeupError;
pub use repository::CodeupRepository;
