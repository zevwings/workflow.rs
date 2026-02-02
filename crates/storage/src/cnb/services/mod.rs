//! Pull Request 服务模块
//!
//! 提供 Pull Request 相关的业务逻辑，按功能拆分为多个子服务：
//! - query: 查询相关功能
//! - mutation: 变更相关功能（创建、更新、关闭、合并）
//! - review: 评论和审批功能
//! - diff: Diff 相关功能
//! - context: 服务共用上下文

mod context;
mod diff;
mod mutation;
mod query;
mod review;

pub use context::{ServiceContext, ServiceContextImpl};
pub use diff::{PullRequestDiffService, PullRequestDiffServiceImpl};
pub use mutation::{PullRequestMutationService, PullRequestMutationServiceImpl};
pub use query::{PullRequestQueryService, PullRequestQueryServiceImpl};
pub use review::{PullRequestReviewService, PullRequestReviewServiceImpl};
