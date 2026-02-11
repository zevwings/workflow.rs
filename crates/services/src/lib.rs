//! 服务层（Application Service Layer）
//!
//! 组合 storage，实现业务用例与领域服务接口。
//!
//! # 对外 API
//!
//! 外部仅通过 [`register_services`] 注册到依赖注入容器，通过 trait 接口访问实现，不直接依赖本 crate 的其他类型。

pub(crate) mod alias;
pub(crate) mod bootstrap;
pub(crate) mod branch;
pub(crate) mod commit;
pub(crate) mod completion;
pub(crate) mod path;
pub(crate) mod pull_request;
pub(crate) mod summary;

#[cfg(any(test, feature = "testing"))]
pub mod testing;

pub use bootstrap::register_services;
