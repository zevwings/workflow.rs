//! 测试相关命令模块
//!
//! 提供测试检查、覆盖率检查、指标收集、报告生成等功能。

// TODO: 按照优先级实现命令
// P0: report::generate
// P1: metrics::collect, report::generate_with_comment, docs::check
// P2: trends::analyze

pub mod check;
pub mod docs;
pub mod metrics;
pub mod report;
pub mod trends;

pub use check::TestsCoverageCheckCommand;
pub use docs::TestsDocsCheckCommand;
pub use metrics::TestsMetricsCollectCommand;
pub use report::TestsReportGenerateCommand;
pub use trends::TestsTrendsAnalyzeCommand;

