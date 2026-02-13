//! Commit 分析子模块
//!
//! 包含提交总结服务实现，以及阶段一/二/三的子服务与 prompt。

mod batch;
mod classify;
mod config;
mod service;
// 允许模块嵌套：summary/summary 模块用于阶段三的总结分析
mod logic;
#[allow(clippy::module_inception)]
mod summary;
mod test_analyze;
mod utils;

pub(crate) use batch::BatchAnalyzeService;
pub(crate) use classify::FileClassifyService;
pub(crate) use config::ConfigAnalyzeService;
pub(crate) use logic::LogicAnalyzeService;
pub(crate) use service::CommitSummaryServiceImpl;
pub(crate) use summary::{SummaryAnalyzeInput, SummaryAnalyzeService};
pub(crate) use test_analyze::TestAnalyzeService;
pub(crate) use utils::{
    compress_diff, format_file_summary, prefilter_files_for_large_commits,
    sample_files_by_change_volume, SamplingConfig,
};
