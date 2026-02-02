//! CNB 业务域
//!
//! 包含 CNB 相关的实体、仓储接口和错误类型

pub mod context;
pub mod entity;
pub mod error;
pub mod repository;

// Re-export public types
pub use context::CNBContext;
pub use entity::CNBUser;
pub use error::CNBError;
pub use repository::CNBRepository;
