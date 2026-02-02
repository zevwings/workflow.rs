//! 全局配置模块
//!
//! 提供全局配置的存储和验证服务实现。

pub mod repository;
pub mod verification_service;

pub use repository::GlobalConfigRepositoryImpl;
pub use verification_service::VerificationServiceImpl;
