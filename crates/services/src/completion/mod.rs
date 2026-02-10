//! Shell Completion 服务实现
//!
//! 实现 `CompletionService` trait，提供 Shell Completion 的配置管理功能。

mod service;

pub(crate) use service::CompletionServiceImpl;
