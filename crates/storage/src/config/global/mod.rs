//! 全局配置模块
//!
//! 提供全局配置的存储和验证服务实现。

mod repository;
mod verification_service;

pub(crate) use repository::GlobalConfigRepositoryImpl;
pub(crate) use verification_service::VerificationServiceImpl;
