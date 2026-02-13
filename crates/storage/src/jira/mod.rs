//! Jira 相关模块
//!
//! 本模块提供了与 Jira REST API 交互的完整功能，包括：
//! - 用户信息管理（获取、缓存）
//! - Ticket/Issue 操作（查询、状态更新、分配、评论）
//! - 项目状态管理（获取状态列表、配置状态映射）
//! - 工作历史记录管理（PR 创建/合并记录）
//! - 数据模型定义（Issue、User、Attachment 等）
//! - 辅助工具函数（字符串处理、认证、URL 构建）
//!
//! ## 架构设计
//!
//! 本模块通过 `JiraConfigProvider` trait 实现依赖倒置，使 Jira 模块独立于具体的配置实现。
//! 配置适配器在 `infra::adapters::config` 模块中实现，将 `Settings` 适配为配置提供者。
//!
//! ## 模块结构
//!
//! - `client` - 统一 HTTP 请求处理客户端（JiraClient）
//! - `api` - 远程 Jira API 交互领域
//!   - `repository` - JiraRepositoryImpl（实现 JiraRepository trait）
//!   - `services` - Issue、Status、User 服务
//! - `history` - 本地工作历史记录领域
//!   - `repository` - JiraWorkHistoryRepositoryImpl（实现 JiraWorkHistoryRepository trait）
//!   - `services` - WorkHistory 服务
//! - `types` - 数据模型定义（API 响应类型）

mod api;
// mod client;
mod history;

// 仅在本 crate 内（registry）使用
pub(crate) use api::{
    AttachmentService, AttachmentServiceImpl, IssueService, IssueServiceImpl, JiraRepositoryImpl,
    StatusService, StatusServiceImpl, UserService, UserServiceImpl,
};
// pub(crate) use client::{JiraClient, JiraClientImpl};
pub(crate) use history::{
    JiraWorkHistoryRepositoryImpl, WorkHistoryService, WorkHistoryServiceImpl,
};
