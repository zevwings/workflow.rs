//! 服务层（Application Service Layer）
//!
//! 组合 storage，实现业务用例
//! 实现领域服务接口

pub mod alias;
pub mod completion;
pub mod pull_request;
pub mod registry;

// Re-export public types
pub use alias::AliasServiceImpl;
pub use completion::CompletionServiceImpl;
pub use pull_request::PullRequestServiceImpl;

// Re-export registry
pub use registry::{register_services, ServicesModule};
