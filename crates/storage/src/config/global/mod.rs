//! 全局配置模块
//!
//! 提供全局配置的存储和验证服务实现。

mod repository;

pub(crate) use repository::GlobalConfigRepositoryImpl;
