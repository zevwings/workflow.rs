//! workflow commit summary：三阶段提交分析（阶段一文件分类 → 阶段二分类分析 → 阶段三全局总结）
//!
//! 对当前分支相对基准分支的变更执行完整分析，输出 Markdown 格式的 commit 总结。
//!
//! 通过 `CommitSummaryService` 委托给 services 层的三阶段分析实现。

use prompt::info;

use crate::registry::get_commit_summary_service;

/// 三阶段提交分析命令（阶段一分类 → 阶段二分析 → 阶段三总结）
pub struct CommitSummaryCommand;

impl Default for CommitSummaryCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl CommitSummaryCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("starting 3-stage commit summary analysis...");

        let summary_service = get_commit_summary_service();
        let summary = summary_service.run_analysis(None)?;

        info!("3-stage commit summary analysis completed.");
        println!("{}", summary.to_markdown());
        Ok(())
    }
}
