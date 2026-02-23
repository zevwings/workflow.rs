//! Jira 工作历史记录领域模块
//!
//! 提供本地工作历史记录管理功能，包括：
//! - PR 创建记录
//! - PR 合并记录
//! - 分支与 PR 关联查询

mod repository;
pub mod services;

pub use repository::JiraWorkHistoryRepositoryImpl;
pub use services::{WorkHistoryService, WorkHistoryServiceImpl};
