//! Jira API 领域模块
//!
//! 提供与远程 Jira REST API 交互的功能，包括：
//! - Issue 操作（查询、状态更新、评论）
//! - 用户信息管理
//! - 项目状态管理
//! - 附件处理

mod repository;
pub mod services;

pub use repository::JiraRepositoryImpl;
pub use services::{
    AttachmentService, AttachmentServiceImpl, IssueService, IssueServiceImpl, StatusService,
    StatusServiceImpl, UserService, UserServiceImpl,
};
