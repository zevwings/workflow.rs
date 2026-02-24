//! Codeup Pull Request 变更服务

use std::sync::Arc;

use domain::CodeupError;

use crate::codeup::{
    services::ServiceContext,
    types::{
        CreatePullRequestRequest, CreatePullRequestResponse, MergePullRequestRequest,
        UpdatePullRequestRequest,
    },
};
use client::CodeupClient;

/// Pull Request 变更服务接口
pub trait PullRequestMutationService: Send + Sync {
    /// 创建 Pull Request
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<String, CodeupError>;

    /// 合并 Pull Request
    fn merge_pull_request(&self, pr_id: &str, force: bool) -> Result<(), CodeupError>;

    /// 关闭 Pull Request
    fn close_pull_request(&self, pr_id: &str) -> Result<(), CodeupError>;

    /// 更新 Pull Request 的标题和/或描述
    fn update_pull_request(
        &self,
        pr_id: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<(), CodeupError>;
}

/// Pull Request 变更服务实现
pub struct PullRequestMutationServiceImpl {
    client: Arc<dyn CodeupClient>,
    context: Arc<dyn ServiceContext>,
}

impl PullRequestMutationServiceImpl {
    pub fn new(client: Arc<dyn CodeupClient>, context: Arc<dyn ServiceContext>) -> Self {
        Self { client, context }
    }
}

impl PullRequestMutationService for PullRequestMutationServiceImpl {
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<String, CodeupError> {
        let project_id = self.context.get_project_id()?;
        let url = format!("/api/v3/projects/{}/code_reviews", project_id);

        let request = CreatePullRequestRequest {
            title: title.to_string(),
            description: body.to_string(),
            source_branch: source_branch.to_string(),
            target_branch: target_branch.to_string(),
        };

        let body = serde_json::to_value(&request)
            .map_err(|e| CodeupError::ApiError(format!("序列化请求失败: {}", e)))?;
        let response = self.client.post(&url, &body)?;
        let json_value = response
            .json()
            .map_err(|e| CodeupError::ApiError(format!("解析响应 JSON 失败: {}", e)))?;
        let response_data: CreatePullRequestResponse = serde_json::from_value(json_value)
            .map_err(|e| CodeupError::ApiError(format!("反序列化响应失败: {}", e)))?;

        Ok(response_data.web_url)
    }

    fn merge_pull_request(&self, pr_id: &str, force: bool) -> Result<(), CodeupError> {
        let project_id = self.context.get_project_id()?;
        let pr_iid = self.context.parse_pr_id(pr_id)?;

        let url = format!(
            "/api/v3/projects/{}/code_reviews/{}/merge",
            project_id, pr_iid
        );

        let request = MergePullRequestRequest {
            merge_commit_message: None,
            should_remove_source_branch: force,
        };

        let body = serde_json::to_value(&request)
            .map_err(|e| CodeupError::ApiError(format!("序列化请求失败: {}", e)))?;
        self.client.post(&url, &body)?;

        Ok(())
    }

    fn close_pull_request(&self, pr_id: &str) -> Result<(), CodeupError> {
        let project_id = self.context.get_project_id()?;
        let pr_iid = self.context.parse_pr_id(pr_id)?;

        let url = format!("/api/v3/projects/{}/code_reviews/{}", project_id, pr_iid);

        let request = UpdatePullRequestRequest {
            title: None,
            description: None,
            state: Some("closed".to_string()),
        };

        let body = serde_json::to_value(&request)
            .map_err(|e| CodeupError::ApiError(format!("序列化请求失败: {}", e)))?;
        self.client.patch(&url, &body)?;

        Ok(())
    }

    fn update_pull_request(
        &self,
        pr_id: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<(), CodeupError> {
        let project_id = self.context.get_project_id()?;
        let pr_iid = self.context.parse_pr_id(pr_id)?;

        let url = format!("/api/v3/projects/{}/code_reviews/{}", project_id, pr_iid);

        let request = UpdatePullRequestRequest {
            title: title.map(|s| s.to_string()),
            description: body.map(|s| s.to_string()),
            state: None,
        };

        let body = serde_json::to_value(&request)
            .map_err(|e| CodeupError::ApiError(format!("序列化请求失败: {}", e)))?;
        self.client.patch(&url, &body)?;

        Ok(())
    }
}
