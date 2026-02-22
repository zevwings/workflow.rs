//! 列出 Pull Requests 命令

use prompt::info;

use crate::bootstrap;

/// Pull Request List 命令
pub struct PullRequestListCommand {
    state: Option<String>,
    limit: Option<usize>,
}

impl PullRequestListCommand {
    /// 创建新的 PullRequestListCommand
    pub fn new(state: Option<String>, limit: Option<usize>) -> Self {
        Self { state, limit }
    }

    /// 运行 `workflow pr list` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pr_service = bootstrap::get_pull_request_service();

        // 列出 Pull Requests
        let prs = pr_service
            .list_pull_requests(self.state.as_deref(), self.limit)
            .map_err(|e| format!("Failed to list Pull Requests: {}", e))?;

        if prs.is_empty() {
            info!("No Pull Requests found");
            return Ok(());
        }

        // 显示 PR 列表
        info!("Pull Requests:");
        for pr in prs {
            let status = if pr.status.merged {
                "merged"
            } else {
                &pr.status.state
            };
            println!("  #{} - {} [{}]", pr.id, pr.title, status);
        }

        Ok(())
    }
}
