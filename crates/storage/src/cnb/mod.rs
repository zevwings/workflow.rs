//! CNB 存储实现
//!
//! 本模块提供了与 CNB API 交互的完整功能，包括：
//! - Pull Request 操作（创建、合并、查询、更新等）
//! - 用户信息管理
//! - 仓库信息查询
//! - 错误处理
//!
//! ## 架构设计
//!
//! 本模块采用服务层架构：
//! - `client/` - CNB API 客户端和配置上下文
//! - `services/` - 内部服务层，包含具体业务逻辑
//! - `repository.rs` - Repository 实现，作为薄委托层

pub(crate) mod client;
pub(crate) mod repository;
pub(crate) mod services;
pub(crate) mod types;

// 重新导出主要类型
pub use client::{CNBClient, CNBClientImpl, CNBContextImpl};
pub use repository::CNBRepositoryImpl;
pub use services::{
    PullRequestDiffService, PullRequestDiffServiceImpl, PullRequestMutationService,
    PullRequestMutationServiceImpl, PullRequestQueryService, PullRequestQueryServiceImpl,
    PullRequestReviewService, PullRequestReviewServiceImpl, ServiceContext, ServiceContextImpl,
};
