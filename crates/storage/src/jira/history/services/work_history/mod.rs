//! 工作历史记录服务模块
//!
//! 提供 PR 创建和合并的工作历史记录管理功能。

mod service;

pub use service::{WorkHistoryService, WorkHistoryServiceImpl};
