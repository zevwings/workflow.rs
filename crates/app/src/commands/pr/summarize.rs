//! 总结 Pull Request 命令

use crate::registry;
use color_eyre::Result;
use prompt::{info, spinner, success};

/// Pull Request Summarize 命令
pub struct PullRequestSummarizeCommand {
    pr_id: Option<String>,
}

impl PullRequestSummarizeCommand {
    /// 创建新的 PullRequestSummarizeCommand
    pub fn new(pr_id: Option<String>) -> Self {
        Self { pr_id }
    }

    /// 运行 `workflow pr summarize` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pr_service = registry::get_pull_request_service();

        // 总结 PR
        info!("Generating Pull Request summary...");
        let summary = spinner!("Generating summary using LLM...")
            .with(|| pr_service.summarize_pull_request(self.pr_id.as_deref()))
            .map_err(|e| format!("Failed to summarize Pull Request: {}", e))?;

        success!("Pull Request summary generated successfully!");
        info!("Filename: {}", summary.filename);
        println!("\n{}", summary.summary);

        Ok(())
    }
}
