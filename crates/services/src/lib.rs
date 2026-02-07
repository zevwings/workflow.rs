//! 服务层（Application Service Layer）
//!
//! 组合 storage，实现业务用例
//! 实现领域服务接口

pub(crate) mod alias;
pub(crate) mod branch;
pub(crate) mod completion;
pub(crate) mod path;
pub(crate) mod pull_request;
pub(crate) mod registry;
pub(crate) mod summary;

pub use branch::BranchServiceImpl;
pub use registry::register_services;
pub use summary::CommitSummaryServiceImpl;