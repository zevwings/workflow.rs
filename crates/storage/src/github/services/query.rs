//! Pull Request 查询服务
//!
//! 提供 Pull Request 查询相关的业务逻辑实现

use std::{fmt::Write, sync::Arc};

use domain::GitHubError;
use toolkit::log_debug;

use crate::github::client::GitHubClient;
use crate::github::services::ServiceContext;
use crate::github::types::{GitHubUserInfo, PullRequestInfo};

/// Pull Request 查询服务接口
pub trait PullRequestQueryService: Send + Sync {
    /// 获取 PR 信息
    fn get_pull_request_info(&self, pull_request_id: &str) -> Result<String, GitHubError>;

    /// 获取 PR URL
    fn get_pull_request_url(&self, pull_request_id: &str) -> Result<String, GitHubError>;

    /// 获取 PR 标题
    fn get_pull_request_title(&self, pull_request_id: &str) -> Result<String, GitHubError>;

    /// 获取 PR body 内容
    fn get_pull_request_body(&self, pull_request_id: &str) -> Result<Option<String>, GitHubError>;

    /// 获取 PR 状态
    fn get_pull_request_status(
        &self,
        pull_request_id: &str,
    ) -> Result<(String, bool, Option<String>), GitHubError>;

    /// 获取 PR 列表
    fn get_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PullRequestInfo>, GitHubError>;

    /// 获取当前分支的 PR ID
    fn get_current_branch_pull_request(
        &self,
        current_branch: &str,
    ) -> Result<Option<String>, GitHubError>;

    /// 获取 PR 信息（内部方法）
    fn fetch_pr_info(&self, pr_number: u64) -> Result<PullRequestInfo, GitHubError>;

    /// 获取 GitHub 用户信息
    fn get_user_info(&self) -> Result<GitHubUserInfo, GitHubError>;
}

/// Pull Request 查询服务实现
pub struct PullRequestQueryServiceImpl {
    client: Arc<dyn GitHubClient>,
    context: Arc<dyn ServiceContext>,
}

impl PullRequestQueryServiceImpl {
    pub fn new(client: Arc<dyn GitHubClient>, context: Arc<dyn ServiceContext>) -> Self {
        Self { client, context }
    }
}

impl PullRequestQueryService for PullRequestQueryServiceImpl {
    fn get_pull_request_info(&self, pull_request_id: &str) -> Result<String, GitHubError> {
        let pr_number = self.context.parse_pr_number(pull_request_id)?;
        let pr = self.fetch_pr_info(pr_number)?;

        let mut info = String::new();
        writeln!(info, "Title: {}", pr.title)
            .map_err(|e| GitHubError::ApiError(format!("Failed to write info: {}", e)))?;
        if let Some(body) = pr.body {
            writeln!(info, "Description: {}", body)
                .map_err(|e| GitHubError::ApiError(format!("Failed to write info: {}", e)))?;
        }
        writeln!(info, "State: {}", pr.state)
            .map_err(|e| GitHubError::ApiError(format!("Failed to write info: {}", e)))?;
        writeln!(info, "Source Branch: {}", pr.head.ref_name)
            .map_err(|e| GitHubError::ApiError(format!("Failed to write info: {}", e)))?;
        writeln!(info, "Target Branch: {}", pr.base.ref_name)
            .map_err(|e| GitHubError::ApiError(format!("Failed to write info: {}", e)))?;
        writeln!(info, "URL: {}", pr.html_url)
            .map_err(|e| GitHubError::ApiError(format!("Failed to write info: {}", e)))?;

        Ok(info)
    }

    fn get_pull_request_url(&self, pull_request_id: &str) -> Result<String, GitHubError> {
        let pr_number = self.context.parse_pr_number(pull_request_id)?;
        let pr = self.fetch_pr_info(pr_number)?;
        Ok(pr.html_url)
    }

    fn get_pull_request_title(&self, pull_request_id: &str) -> Result<String, GitHubError> {
        let pr_number = self.context.parse_pr_number(pull_request_id)?;
        let pr = self.fetch_pr_info(pr_number)?;
        Ok(pr.title)
    }

    fn get_pull_request_body(&self, pull_request_id: &str) -> Result<Option<String>, GitHubError> {
        let pr_number = self.context.parse_pr_number(pull_request_id)?;
        let pr = self.fetch_pr_info(pr_number)?;
        Ok(pr.body)
    }

    fn get_pull_request_status(
        &self,
        pull_request_id: &str,
    ) -> Result<(String, bool, Option<String>), GitHubError> {
        let pr_number = self.context.parse_pr_number(pull_request_id)?;
        let pr = self.fetch_pr_info(pr_number)?;
        Ok((pr.state, pr.merged, pr.merged_at))
    }

    fn get_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PullRequestInfo>, GitHubError> {
        let (owner, repo_name) = self.context.get_owner_and_repo()?;

        let state = match state {
            Some("open") => "open",
            Some("closed") => "closed",
            Some("merged") => "closed",
            Some("all") | None => "all",
            _ => "all",
        };
        let per_page = limit.unwrap_or(30).min(100);

        let url = format!(
            "/repos/{}/{}/pulls?state={}&per_page={}",
            owner, repo_name, state, per_page
        );

        let response = self.client.get(&url)?;
        let json_value = response
            .as_json()
            .map_err(|e| GitHubError::ApiError(format!("Failed to parse response JSON: {}", e)))?;
        let prs: Vec<PullRequestInfo> = serde_json::from_value(json_value)
            .map_err(|e| GitHubError::ApiError(format!("Failed to deserialize PR list: {}", e)))?;

        Ok(prs)
    }

    fn get_current_branch_pull_request(
        &self,
        current_branch: &str,
    ) -> Result<Option<String>, GitHubError> {
        let (owner, repo_name) = self.context.get_owner_and_repo()?;

        for state in ["open", "all"] {
            let url = format!(
                "/repos/{}/{}/pulls?head={}:{}&state={}",
                owner, repo_name, owner, current_branch, state
            );

            let response = self.client.get(&url)?;
            let json_value = response.as_json().map_err(|e| {
                GitHubError::ApiError(format!("Failed to parse response JSON: {}", e))
            })?;
            let prs: Vec<PullRequestInfo> = serde_json::from_value(json_value).map_err(|e| {
                GitHubError::ApiError(format!("Failed to deserialize PR list: {}", e))
            })?;

            if let Some(pr) = prs.first() {
                if state == "all" {
                    log_debug!(
                        "Found PR #{} for branch '{}' (state: {})",
                        pr.number,
                        current_branch,
                        pr.state
                    );
                }
                return Ok(Some(pr.number.to_string()));
            }
        }

        Ok(None)
    }

    fn fetch_pr_info(&self, pr_number: u64) -> Result<PullRequestInfo, GitHubError> {
        let (owner, repo_name) = self.context.get_owner_and_repo()?;

        let url = format!("/repos/{}/{}/pulls/{}", owner, repo_name, pr_number);

        let response = self.client.get(&url)?;
        let json_value = response
            .as_json()
            .map_err(|e| GitHubError::ApiError(format!("Failed to parse response JSON: {}", e)))?;
        let pr_info: PullRequestInfo = serde_json::from_value(json_value)
            .map_err(|e| GitHubError::ApiError(format!("Failed to deserialize PR info: {}", e)))?;
        Ok(pr_info)
    }

    fn get_user_info(&self) -> Result<GitHubUserInfo, GitHubError> {
        let url = "/user";
        let response = self.client.get(url)?;
        let json_value = response
            .as_json()
            .map_err(|e| GitHubError::ApiError(format!("Failed to parse response JSON: {}", e)))?;
        let user: GitHubUserInfo = serde_json::from_value(json_value).map_err(|e| {
            GitHubError::ApiError(format!("Failed to deserialize user info: {}", e))
        })?;

        Ok(user)
    }
}
