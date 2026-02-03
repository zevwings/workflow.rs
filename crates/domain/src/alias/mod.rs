//! 别名业务域
//!
//! 包含别名管理相关的实体和服务接口

pub mod entity;
pub mod service;

// Re-export public types
pub use entity::{AliasAddResult, AliasInfo, AliasListResult, AliasRemoveResult};
pub use service::AliasService;
