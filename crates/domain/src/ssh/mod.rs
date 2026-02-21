//! SSH 业务域
//!
//! 包含 SSH 密钥管理的实体、服务接口和错误类型

pub mod entity;
pub mod error;
pub mod service;

pub use entity::SshKeyInfo;
pub use error::SshError;
pub use service::SshService;
