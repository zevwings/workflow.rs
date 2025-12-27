//! 测试数据管理
//!
//! 提供测试数据生成、缓存、清理和版本管理的完整功能。

pub mod cache;
pub mod cleanup;
pub mod factory;
pub mod version;

// 重新导出常用类型，保持向后兼容
pub use factory::TestDataFactory;
