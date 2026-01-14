//! GitHub 适配器
//!
//! 将 `github::GitHub` 适配为 `PlatformProvider` trait，实现依赖倒置。
//! 使 PR 模块可以通过适配器使用 GitHub API，而不直接依赖 GitHub 实现。

use crate::domain::pr::platform::{PlatformProvider, PullRequestStatus};
use crate::domain::pr::PullRequestRow;
use crate::services::git::GitRepo;
use crate::services::github::GitHub;
use crate::services::jira::history::JiraWorkHistory;
use color_eyre::Result;

/// GitHub 平台适配器
///
/// 将 `github::GitHub` 适配为 `PlatformProvider` trait。
pub struct GitHubAdapter {
    github: GitHub,
}

impl GitHubAdapter {
    /// 创建新的适配器实例
    pub fn new() -> Self {
        Self { github: GitHub }
    }
}

impl Default for GitHubAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformProvider for GitHubAdapter {
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: Option<&str>,
    ) -> Result<String> {
        self.github.create_pull_request(title, body, source_branch, target_branch)
    }

    fn merge_pull_request(&self, pull_request_id: &str, delete_branch: bool) -> Result<()> {
        self.github.merge_pull_request(pull_request_id, delete_branch)
    }

    fn get_pull_request_info(&self, pull_request_id_or_branch: &str) -> Result<String> {
        self.github.get_pull_request_info(pull_request_id_or_branch)
    }

    fn get_pull_request_url(&self, pull_request_id: &str) -> Result<String> {
        self.github.get_pull_request_url(pull_request_id)
    }

    fn get_pull_request_title(&self, pull_request_id: &str) -> Result<String> {
        self.github.get_pull_request_title(pull_request_id)
    }

    fn get_pull_request_body(&self, pull_request_id: &str) -> Result<Option<String>> {
        self.github.get_pull_request_body(pull_request_id)
    }

    fn get_current_branch_pull_request(&self) -> Result<Option<String>> {
        // 首先尝试通过 GitHub API 查找
        if let Ok(Some(pr_id)) = self.github.get_current_branch_pull_request() {
            return Ok(Some(pr_id));
        }

        // 如果 API 查询没有找到，尝试从 work-history 文件中查找
        let current_branch = crate::git::GitBranch::current_branch()?;
        let remote_url = GitRepo::get_remote_url().ok();
        if let Some(pr_id) =
            JiraWorkHistory::find_pr_id_by_branch(&current_branch, remote_url.as_deref())?
        {
            crate::log_debug!(
                "Found PR #{} for branch '{}' from work-history",
                pr_id,
                current_branch
            );
            return Ok(Some(pr_id));
        }

        Ok(None)
    }

    fn get_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PullRequestRow>> {
        let prs = self.github.get_pull_requests(state, limit)?;
        let rows: Vec<PullRequestRow> = prs
            .into_iter()
            .map(|pr| PullRequestRow {
                number: pr.number.to_string(),
                state: pr.state,
                branch: pr.head.ref_name,
                title: pr.title,
                author: pr
                    .user
                    .as_ref()
                    .map(|u| u.login.clone())
                    .unwrap_or_else(|| "N/A".to_string()),
                url: pr.html_url,
            })
            .collect();
        Ok(rows)
    }

    fn get_pull_request_status(&self, pull_request_id: &str) -> Result<PullRequestStatus> {
        let (state, merged, merged_at) = self.github.get_pull_request_status(pull_request_id)?;
        Ok(PullRequestStatus {
            state,
            merged,
            merged_at,
        })
    }

    fn close_pull_request(&self, pull_request_id: &str) -> Result<()> {
        self.github.close_pull_request(pull_request_id)
    }

    fn get_pull_request_diff(&self, pull_request_id: &str) -> Result<String> {
        self.github.get_pull_request_diff(pull_request_id)
    }

    fn add_comment(&self, pull_request_id: &str, comment: &str) -> Result<()> {
        self.github.add_comment(pull_request_id, comment)
    }

    fn approve_pull_request(&self, pull_request_id: &str) -> Result<()> {
        self.github.approve_pull_request(pull_request_id)
    }

    fn update_pr_base(&self, pull_request_id: &str, new_base: &str) -> Result<()> {
        self.github.update_pr_base(pull_request_id, new_base)
    }

    fn update_pull_request(
        &self,
        pull_request_id: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<()> {
        self.github.update_pull_request(pull_request_id, title, body)
    }
}
