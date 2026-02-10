//! 存储实现层（Storage Adapters）
//!
//! 实现各种仓储接口，提供数据持久化和外部服务调用。
//!
//! # 导出可见性
//!
//! - **`pub`**：对外公开。仅 [`register_storage`] 与 `git` 模块（供 examples/benches 使用）。
//! - **`pub(crate)`**：仅本 crate 内可见。`config`、`github`、`jira`、`registry` 及各自 re-export。
//! - **`pub(super)`**：仅父模块可见。如 registry 子模块中的 `register_*` 函数。
//! - **私有**：仅当前模块可见。各子模块内部实现。
//!
//! 业务代码（如 app）仅通过 [`register_storage`] 注册到依赖注入容器，通过 trait 接口访问实现。

pub mod git;

pub(crate) mod config;
pub(crate) mod github;
pub(crate) mod jira;
pub(crate) mod registry;

pub use registry::register_storage;
