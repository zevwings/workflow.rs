//! 存储实现层（Storage Adapters）
//!
//! 实现各种仓储接口，提供数据持久化和外部服务调用

pub(crate) mod config;
pub(crate) mod git;
pub(crate) mod github;
pub(crate) mod jira;
pub(crate) mod llm;
pub(crate) mod registry;
// 导出服务注册函数
pub use registry::register_storage;
