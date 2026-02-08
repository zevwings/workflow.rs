//! 存储实现层（Storage Adapters）
//!
//! 实现各种仓储接口，提供数据持久化和外部服务调用
//!
//! # 模块导出策略
//!
//! - **`git`**: 公开导出，提供 Git 操作的核心 API（`GitContext`, `GitRepositoryImpl`）
//!   - 外部需要直接使用这些类型进行 Git 仓库操作
//!   - 包含测试工具模块（`testing`, `performance`）
//!
//! - **其他模块**（`config`, `github`, `jira`）: 内部实现
//!   - 仅通过 [`register_storage`] 函数注册到依赖注入容器
//!   - 外部通过 trait 接口访问，无需直接依赖实现类型

// Git 模块 - 公开导出以提供核心 Git 操作 API
pub mod git;

// 内部实现模块 - 通过依赖注入访问
pub(crate) mod config;
pub(crate) mod github;
pub(crate) mod jira;
pub(crate) mod registry;

// 导出服务注册函数
pub use registry::register_storage;
