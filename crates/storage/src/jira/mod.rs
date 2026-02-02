//! Jira 相关模块
//!
//! 本模块提供了与 Jira REST API 交互的完整功能，包括：
//! - 用户信息管理（获取、缓存）
//! - Ticket/Issue 操作（查询、状态更新、分配、评论）
//! - 项目状态管理（获取状态列表、配置状态映射）
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
//! - `provider` - JiraConfigProvider trait（配置提供者接口）
//! - `client` - 统一 HTTP 请求处理客户端（JiraClient）
//! - `repository` - 仓储实现（JiraRepositoryImpl）
//! - `types` - 数据模型定义（API 响应类型）
//! - `helpers` - 辅助函数（字符串处理、认证、URL 构建）
//! - `paths` - Jira 附件路径管理

pub(crate) mod client;
pub(crate) mod repository;
pub(crate) mod services;
pub(crate) mod types;

// 重新导出 client
pub use client::{JiraClient, JiraClientImpl, JiraConfigContextImpl};

// 重新导出 repository 实现
pub use repository::JiraRepositoryImpl;
pub use services::{
    IssueService, IssueServiceImpl, StatusService, StatusServiceImpl, UserService, UserServiceImpl,
};
