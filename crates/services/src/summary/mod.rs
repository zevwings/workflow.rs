//! Commit 分析子模块
//!
//! 包含提交总结服务实现，以及阶段一/二/三的子服务与 prompt。

mod batch;
mod classify;
mod config;
mod prompt;
mod service;
mod summary;
mod tests;
mod logic;

pub(crate) use batch::BatchAnalyzeService;
pub(crate) use classify::FileClassifyService;
pub(crate) use config::ConfigAnalyzeService;
pub(crate) use logic::LogicAnalyzeService;
pub(crate) use summary::{SummaryAnalyzeInput, SummaryAnalyzeService};
pub(crate) use tests::TestAnalyzeService;

pub use service::CommitSummaryServiceImpl;
