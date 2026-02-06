//! 服务层（Application Service Layer）
//!
//! 组合 storage，实现业务用例
//! 实现领域服务接口

pub(crate) mod alias;
pub(crate) mod completion;
pub(crate) mod path;
pub(crate) mod pull_request;
pub(crate) mod registry;

// Re-export public types
pub(crate) use alias::AliasServiceImpl;
pub(crate) use completion::CompletionServiceImpl;
pub(crate) use pull_request::PullRequestServiceImpl;

// Re-export registry
pub use registry::register_services;
