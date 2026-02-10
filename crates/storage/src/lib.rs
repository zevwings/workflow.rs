//! 存储实现层（Storage Adapters）
//!
//! 实现各种仓储接口，提供数据持久化和外部服务调用。
//!
//! # 对外 API
//!
//! 业务代码（如 app）仅通过 [`register_storage`] 注册到依赖注入容器，通过 trait 接口访问实现。
//!
//! - **`git`** 为 `pub`，供本 crate 的 examples 与 benches 使用（二者为独立编译单元，需公开 API）。
//! - 其余模块为内部实现，仅通过 `register_storage` 暴露。

pub mod git;

pub(crate) mod config;
pub(crate) mod github;
pub(crate) mod jira;
pub(crate) mod registry;

// 导出服务注册函数
pub use registry::register_storage;
