//! Dev 工具命令模块
//!
//! 提供文档检查、测试检查、报告生成等开发工具功能。

// TODO: 按照优先级实现命令模块
// P0: tests::check::coverage, tests::report::generate
// P1: tests::metrics, tests::report::comment, tests::check::doctests
// P2: 其他命令

pub mod checksum;
pub mod ci;
pub mod docs;
pub mod homebrew;
pub mod performance;
pub mod pr;
pub mod tag;
pub mod tests;
pub mod version;

pub use checksum::ChecksumCalculateCommand;
pub use ci::{CiSkipCommand, CiVerifyCommand};
pub use docs::{DocsIntegrityCheckCommand, DocsLinksCheckCommand, DocsReportGenerateCommand};
pub use homebrew::HomebrewUpdateCommand;
pub use performance::PerformanceAnalyzeCommand;
pub use pr::{PrCreateCommand, PrMergeCommand};
pub use tag::{TagCleanupCommand, TagCreateCommand};
pub use tests::{
    TestsCoverageCheckCommand, TestsDocsCheckCommand, TestsMetricsCollectCommand,
    TestsReportGenerateCommand, TestsTrendsAnalyzeCommand,
};
pub use version::{VersionGenerateCommand, VersionInfo};
