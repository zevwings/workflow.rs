//! Jira 客户端核心模块
//!
//! 本模块提供了统一配置驱动的 Jira REST API 客户端实现。
//!
//! ## 架构设计
//!
//! 本模块通过 `JiraConfigProvider` trait 实现依赖倒置，使 Jira 客户端独立于具体的配置实现。
//! 配置适配器在 `infra::adapters::config` 模块中实现，将 `Settings` 适配为配置提供者。

pub mod core;
pub(crate) mod types;

// 重新导出 API
pub use core::{JiraClient, JiraClientImpl};
