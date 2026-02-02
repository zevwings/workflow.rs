//! Pull Request 变更服务
//!
//! 提供 Pull Request 变更相关的业务逻辑实现，包括创建、更新、关闭、合并等

use std::sync::Arc;

use domain::CNBError;
use serde_json::json;
use toolkit::log_debug;

use crate::cnb::client::CNBClient;
use crate::cnb::services::{PullRequestQueryService, ServiceContext};
use crate::cnb::types::{CreatePullRequest, MergePullRequest, UpdatePullRequest};

/// Pull Request 变更服务接口
pub trait PullRequestMutationService: Send + Sync {
    /// 创建 Pull Request
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<String, CNBError>;

    /// 合并 Pull Request
    fn merge_pull_request(&self, pull_request_id: &str, force: bool) -> Result<(), CNBError>;

    /// 关闭 Pull Request
    fn close_pull_request(&self, pull_request_id: &str) -> Result<(), CNBError>;

    /// 更新 PR 的 base 分支
    fn update_pr_base(&self, pull_request_id: &str, new_base: &str) -> Result<(), CNBError>;

    /// 更新 Pull Request 的标题和/或描述
    fn update_pull_request(
        &self,
        pull_request_id: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<(), CNBError>;
}

/// Pull Request 变更服务实现
pub struct PullRequestMutationServiceImpl {
    client: Arc<dyn CNBClient>,
    query_service: Arc<dyn PullRequestQueryService>,
    context: Arc<dyn ServiceContext>,
}

impl PullRequestMutationServiceImpl {
    pub fn new(
        client: Arc<dyn CNBClient>,
        query_service: Arc<dyn PullRequestQueryService>,
        context: Arc<dyn ServiceContext>,
    ) -> Self {
        Self { client, query_service, context }
    }
}

impl PullRequestMutationService for PullRequestMutationServiceImpl {
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<String, CNBError> {
        let project_path = self.context.project_path()?;

        // 验证项目路径
        log_debug!("Extracted project path: {}", project_path);
        log_debug!("Verifying repository exists at path: {}", project_path);

        // 验证仓库是否存在
        toolkit::log_info!("Verifying repository access...");
        let repo_exists = self.query_service.verify_repository_access()?;
        if !repo_exists {
            return Err(CNBError::NotFound(format!(
                "Repository '{}' not found in your CNB account. Please check:\n\
                1. The git remote URL is correct\n\
                2. You have access to this repository in CNB\n\
                3. The repository exists at https://cnb.cool/{}\n\
                4. Your API token has the necessary permissions",
                project_path, project_path
            )));
        }

        // 提示用户检查仓库设置
        toolkit::log_info!("Creating PR for repository: {}", project_path);
        toolkit::log_info!("Source branch: {} -> Target branch: {}", source_branch, target_branch);

        let encoded_path = urlencoding::encode(&project_path);
        let url = format!("/repos/{}/pulls", encoded_path);

        log_debug!("API endpoint: {}", url);
        log_debug!("Encoded path: {}", encoded_path);

        let request = CreatePullRequest {
            title: title.to_string(),
            body: body.to_string(),
            head: source_branch.to_string(),
            base: target_branch.to_string(),
            head_repo: None,
        };

        log_debug!("Creating PR: {} -> {}", source_branch, target_branch);
        log_debug!("PR Title: {}", title);

        let body = serde_json::to_value(&request)
            .map_err(|e| CNBError::ApiError(format!("Failed to serialize request: {}", e)))?;
        let response = self.client.post(&url, &body)?;
        let pr: crate::cnb::types::PullRequestInfo = response.json()?;

        // 返回 PR URL
        pr.html_url
            .ok_or_else(|| CNBError::ApiError("PR URL not available".to_string()))
    }

    fn merge_pull_request(&self, pull_request_id: &str, _force: bool) -> Result<(), CNBError> {
        let project_path = self.context.project_path()?;
        let encoded_path = urlencoding::encode(&project_path);
        let pr_number = self.context.parse_pr_number(pull_request_id)?;

        let url = format!("/repos/{}/pulls/{}/merge", encoded_path, pr_number);

        // CNB 默认使用 merge 策略
        let request = MergePullRequest {
            merge_style: Some("merge".to_string()),
            commit_title: None,
            commit_message: None,
        };

        log_debug!("Merging PR: {}", pr_number);

        let body = serde_json::to_value(&request)
            .map_err(|e| CNBError::ApiError(format!("Failed to serialize request: {}", e)))?;
        self.client.put(&url, &body)?;

        Ok(())
    }

    fn close_pull_request(&self, pull_request_id: &str) -> Result<(), CNBError> {
        let project_path = self.context.project_path()?;
        let encoded_path = urlencoding::encode(&project_path);
        let pr_number = self.context.parse_pr_number(pull_request_id)?;

        let url = format!("/repos/{}/pulls/{}", encoded_path, pr_number);

        let request = UpdatePullRequest {
            title: None,
            body: None,
            state: Some("closed".to_string()),
        };

        log_debug!("Closing PR: {}", pr_number);

        let body = serde_json::to_value(&request)
            .map_err(|e| CNBError::ApiError(format!("Failed to serialize request: {}", e)))?;
        self.client.patch(&url, &body)?;

        Ok(())
    }

    fn update_pr_base(&self, pull_request_id: &str, new_base: &str) -> Result<(), CNBError> {
        let project_path = self.context.project_path()?;
        let encoded_path = urlencoding::encode(&project_path);
        let pr_number = self.context.parse_pr_number(pull_request_id)?;

        let url = format!("/repos/{}/pulls/{}", encoded_path, pr_number);

        let body = json!({
            "base": new_base,
        });

        log_debug!("Updating PR base to: {}", new_base);

        self.client.patch(&url, &body)?;

        Ok(())
    }

    fn update_pull_request(
        &self,
        pull_request_id: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<(), CNBError> {
        let project_path = self.context.project_path()?;
        let encoded_path = urlencoding::encode(&project_path);
        let pr_number = self.context.parse_pr_number(pull_request_id)?;

        let url = format!("/repos/{}/pulls/{}", encoded_path, pr_number);

        let request = UpdatePullRequest {
            title: title.map(|s| s.to_string()),
            body: body.map(|s| s.to_string()),
            state: None,
        };

        log_debug!("Updating PR: {}", pr_number);

        let body_value = serde_json::to_value(&request)
            .map_err(|e| CNBError::ApiError(format!("Failed to serialize request: {}", e)))?;
        self.client.patch(&url, &body_value)?;

        Ok(())
    }
}
