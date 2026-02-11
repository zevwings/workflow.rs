//! Pull Request 变更服务
//!
//! 提供 Pull Request 变更相关的业务逻辑实现，包括创建、更新、关闭、合并等

use std::sync::Arc;

use domain::GitHubError;
use toolkit::log_debug;

use crate::github::{
    client::GitHubClient,
    services::{PullRequestQueryService, ServiceContext},
    types::{
        CreatePullRequestRequest, CreatePullRequestResponse, MergePullRequestRequest,
        RepositoryInfo, UpdatePullRequestRequest,
    },
};

/// Pull Request 变更服务接口
pub trait PullRequestMutationService: Send + Sync {
    /// 创建 Pull Request
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<String, GitHubError>;

    /// 合并 Pull Request
    fn merge_pull_request(&self, pull_request_id: &str, force: bool) -> Result<(), GitHubError>;

    /// 关闭 Pull Request
    fn close_pull_request(&self, pull_request_id: &str) -> Result<(), GitHubError>;

    /// 更新 PR 的 base 分支
    fn update_pr_base(&self, pull_request_id: &str, new_base: &str) -> Result<(), GitHubError>;

    /// 更新 Pull Request 的标题和/或描述
    fn update_pull_request(
        &self,
        pull_request_id: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<(), GitHubError>;
}

/// Pull Request 变更服务实现
pub struct PullRequestMutationServiceImpl {
    client: Arc<dyn GitHubClient>,
    query_service: Arc<dyn PullRequestQueryService>,
    context: Arc<dyn ServiceContext>,
}

impl PullRequestMutationServiceImpl {
    pub fn new(
        client: Arc<dyn GitHubClient>,
        query_service: Arc<dyn PullRequestQueryService>,
        context: Arc<dyn ServiceContext>,
    ) -> Self {
        Self {
            client,
            query_service,
            context,
        }
    }

    /// 获取仓库信息
    fn get_repository_info(
        &self,
        owner: &str,
        repo_name: &str,
    ) -> Result<RepositoryInfo, GitHubError> {
        let url = format!("/repos/{}/{}", owner, repo_name);
        let response = self.client.get(&url)?;
        let json_value = response.json().map_err(|e| {
            GitHubError::ApiError(format!("Failed to parse repository info JSON: {}", e))
        })?;
        serde_json::from_value(json_value).map_err(|e| {
            GitHubError::ApiError(format!("Failed to deserialize repository info: {}", e))
        })
    }

    /// 获取首选的合并方法：优先使用 squash，其次 rebase，最后 merge
    fn get_preferred_merge_method(
        &self,
        owner: &str,
        repo_name: &str,
    ) -> Result<String, GitHubError> {
        let repo_info = self.get_repository_info(owner, repo_name)?;

        if repo_info.allow_squash_merge.unwrap_or(false) {
            return Ok("squash".to_string());
        }

        if repo_info.allow_rebase_merge.unwrap_or(false) {
            return Ok("rebase".to_string());
        }

        if repo_info.allow_merge_commit.unwrap_or(false) {
            return Ok("merge".to_string());
        }

        Err(GitHubError::ApiError(
            "Repository does not support squash, rebase, or merge commit methods".to_string(),
        ))
    }
}

impl PullRequestMutationService for PullRequestMutationServiceImpl {
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<String, GitHubError> {
        let (owner, repo_name) = self.context.get_owner_and_repo()?;

        let url = format!("/repos/{}/{}/pulls", owner, repo_name);

        let request = CreatePullRequestRequest {
            title: title.to_string(),
            body: body.to_string(),
            head: format!("{}:{}", owner, source_branch),
            base: target_branch.to_string(),
        };

        let body = serde_json::to_value(&request)
            .map_err(|e| GitHubError::ApiError(format!("Failed to serialize request: {}", e)))?;
        let response = self.client.post(&url, &body)?;
        let json_value = response
            .json()
            .map_err(|e| GitHubError::ApiError(format!("Failed to parse response JSON: {}", e)))?;
        let response_data: CreatePullRequestResponse = serde_json::from_value(json_value)
            .map_err(|e| GitHubError::ApiError(format!("Failed to deserialize response: {}", e)))?;

        Ok(response_data.html_url)
    }

    fn merge_pull_request(&self, pull_request_id: &str, force: bool) -> Result<(), GitHubError> {
        let (owner, repo_name) = self.context.get_owner_and_repo()?;
        let pr_number = self.context.parse_pr_number(pull_request_id)?;

        let merge_method = self.get_preferred_merge_method(&owner, &repo_name)?;
        log_debug!("Using merge method: {}", merge_method);

        let url = format!("/repos/{}/{}/pulls/{}/merge", owner, repo_name, pr_number);

        let request = MergePullRequestRequest {
            commit_title: None,
            commit_message: None,
            merge_method,
        };

        let body = serde_json::to_value(&request)
            .map_err(|e| GitHubError::ApiError(format!("Failed to serialize request: {}", e)))?;
        self.client.put(&url, &body)?;

        if force {
            let pr_info = self.query_service.fetch_pr_info(pr_number)?;
            let branch_name = pr_info.head.ref_name;
            let branch_url = format!(
                "/repos/{}/{}/git/refs/heads/{}",
                owner, repo_name, branch_name
            );
            if let Err(e) = self.client.delete(&branch_url) {
                log_debug!("Failed to delete branch {}: {}", branch_name, e);
            }
        }

        Ok(())
    }

    fn close_pull_request(&self, pull_request_id: &str) -> Result<(), GitHubError> {
        let (owner, repo_name) = self.context.get_owner_and_repo()?;
        let pr_number = self.context.parse_pr_number(pull_request_id)?;

        let url = format!("/repos/{}/{}/pulls/{}", owner, repo_name, pr_number);

        let request = UpdatePullRequestRequest {
            title: None,
            body: None,
            state: Some("closed".to_string()),
            base: None,
        };

        let body = serde_json::to_value(&request)
            .map_err(|e| GitHubError::ApiError(format!("Failed to serialize request: {}", e)))?;
        self.client.patch(&url, &body)?;

        Ok(())
    }

    fn update_pr_base(&self, pull_request_id: &str, new_base: &str) -> Result<(), GitHubError> {
        let (owner, repo_name) = self.context.get_owner_and_repo()?;
        let pr_number = self.context.parse_pr_number(pull_request_id)?;

        let url = format!("/repos/{}/{}/pulls/{}", owner, repo_name, pr_number);

        let request = serde_json::json!({
            "base": new_base
        });

        self.client.patch(&url, &request)?;

        Ok(())
    }

    fn update_pull_request(
        &self,
        pull_request_id: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<(), GitHubError> {
        let (owner, repo_name) = self.context.get_owner_and_repo()?;
        let pr_number = self.context.parse_pr_number(pull_request_id)?;

        let url = format!("/repos/{}/{}/pulls/{}", owner, repo_name, pr_number);

        let request = UpdatePullRequestRequest {
            title: title.map(|s| s.to_string()),
            body: body.map(|s| s.to_string()),
            state: None,
            base: None,
        };

        let body = serde_json::to_value(&request)
            .map_err(|e| GitHubError::ApiError(format!("Failed to serialize request: {}", e)))?;
        self.client.patch(&url, &body)?;

        Ok(())
    }
}
