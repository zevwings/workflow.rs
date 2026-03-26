//! Codeup Pull Request 查询服务

use std::{fmt::Write, sync::Arc};

use domain::{CodeupError, CodeupUser};

use crate::codeup::{services::ServiceContext, types::PullRequestInfo};
use client::CodeupClient;

/// Pull Request 查询服务接口
pub trait PullRequestQueryService: Send + Sync {
    /// 获取 PR 信息
    fn get_pull_request_info(&self, pr_id: &str) -> Result<String, CodeupError>;

    /// 获取 PR URL
    fn get_pull_request_url(&self, pr_id: &str) -> Result<String, CodeupError>;

    /// 获取 PR 标题
    fn get_pull_request_title(&self, pr_id: &str) -> Result<String, CodeupError>;

    /// 获取 PR body 内容
    fn get_pull_request_body(&self, pr_id: &str) -> Result<Option<String>, CodeupError>;

    /// 获取 PR 状态
    fn get_pull_request_status(
        &self,
        pr_id: &str,
    ) -> Result<(String, bool, Option<String>), CodeupError>;

    /// 获取 PR 列表
    #[allow(dead_code)]
    fn get_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PullRequestInfo>, CodeupError>;

    /// 获取当前分支的 PR ID
    fn get_current_branch_pull_request(
        &self,
        current_branch: &str,
    ) -> Result<Option<String>, CodeupError>;

    /// 获取 PR 信息（内部方法）
    fn fetch_pr_info(&self, pr_iid: i64) -> Result<PullRequestInfo, CodeupError>;

    /// 获取 Codeup 用户信息
    fn get_user_info(&self) -> Result<CodeupUser, CodeupError>;
}

/// Pull Request 查询服务实现
pub struct PullRequestQueryServiceImpl {
    client: Arc<dyn CodeupClient>,
    context: Arc<dyn ServiceContext>,
}

impl PullRequestQueryServiceImpl {
    pub fn new(client: Arc<dyn CodeupClient>, context: Arc<dyn ServiceContext>) -> Self {
        Self { client, context }
    }
}

impl PullRequestQueryService for PullRequestQueryServiceImpl {
    fn get_pull_request_info(&self, pr_id: &str) -> Result<String, CodeupError> {
        let pr_iid = self.context.parse_pr_id(pr_id)?;
        let pr = self.fetch_pr_info(pr_iid)?;

        let mut info = String::new();
        writeln!(info, "标题: {}", pr.title).ok();
        if let Some(desc) = pr.description {
            writeln!(info, "描述: {}", desc).ok();
        }
        writeln!(info, "状态: {}", pr.state).ok();
        writeln!(info, "源分支: {}", pr.source_branch).ok();
        writeln!(info, "目标分支: {}", pr.target_branch).ok();
        writeln!(info, "URL: {}", pr.web_url).ok();

        Ok(info)
    }

    fn get_pull_request_url(&self, pr_id: &str) -> Result<String, CodeupError> {
        let pr_iid = self.context.parse_pr_id(pr_id)?;
        let pr = self.fetch_pr_info(pr_iid)?;
        Ok(pr.web_url)
    }

    fn get_pull_request_title(&self, pr_id: &str) -> Result<String, CodeupError> {
        let pr_iid = self.context.parse_pr_id(pr_id)?;
        let pr = self.fetch_pr_info(pr_iid)?;
        Ok(pr.title)
    }

    fn get_pull_request_body(&self, pr_id: &str) -> Result<Option<String>, CodeupError> {
        let pr_iid = self.context.parse_pr_id(pr_id)?;
        let pr = self.fetch_pr_info(pr_iid)?;
        Ok(pr.description)
    }

    fn get_pull_request_status(
        &self,
        pr_id: &str,
    ) -> Result<(String, bool, Option<String>), CodeupError> {
        let pr_iid = self.context.parse_pr_id(pr_id)?;
        let pr = self.fetch_pr_info(pr_iid)?;
        Ok((pr.state, pr.merged, None))
    }

    fn get_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PullRequestInfo>, CodeupError> {
        let project_id = self.context.get_project_id()?;

        let state_param = match state {
            Some("open") => "opened",
            Some("closed") => "closed",
            Some("merged") => "merged",
            _ => "all",
        };

        let per_page = limit.unwrap_or(30).min(100);
        let url = format!(
            "/api/v3/projects/{}/code_reviews?state={}&per_page={}",
            project_id, state_param, per_page
        );

        let response = self.client.get(&url)?;
        let json_value = response
            .json()
            .map_err(|e| CodeupError::ApiError(format!("解析响应 JSON 失败: {}", e)))?;
        let prs: Vec<PullRequestInfo> = serde_json::from_value(json_value)
            .map_err(|e| CodeupError::ApiError(format!("反序列化 PR 列表失败: {}", e)))?;

        Ok(prs)
    }

    fn get_current_branch_pull_request(
        &self,
        current_branch: &str,
    ) -> Result<Option<String>, CodeupError> {
        let project_id = self.context.get_project_id()?;

        for state in ["opened", "all"] {
            let url = format!(
                "/api/v3/projects/{}/code_reviews?source_branch={}&state={}",
                project_id, current_branch, state
            );

            let response = self.client.get(&url)?;
            let json_value = response
                .json()
                .map_err(|e| CodeupError::ApiError(format!("解析响应 JSON 失败: {}", e)))?;
            let prs: Vec<PullRequestInfo> = serde_json::from_value(json_value)
                .map_err(|e| CodeupError::ApiError(format!("反序列化 PR 列表失败: {}", e)))?;

            if let Some(pr) = prs.first() {
                return Ok(Some(pr.iid.to_string()));
            }
        }

        Ok(None)
    }

    fn fetch_pr_info(&self, pr_iid: i64) -> Result<PullRequestInfo, CodeupError> {
        let project_id = self.context.get_project_id()?;

        let url = format!("/api/v3/projects/{}/code_reviews/{}", project_id, pr_iid);

        let response = self.client.get(&url)?;
        let json_value = response
            .json()
            .map_err(|e| CodeupError::ApiError(format!("解析响应 JSON 失败: {}", e)))?;
        let pr_info: PullRequestInfo = serde_json::from_value(json_value)
            .map_err(|e| CodeupError::ApiError(format!("反序列化 PR 信息失败: {}", e)))?;
        Ok(pr_info)
    }

    fn get_user_info(&self) -> Result<CodeupUser, CodeupError> {
        // Codeup API 可能需要通过其他方式获取用户信息
        // 这里暂时返回错误，后续根据实际 API 调整
        Err(CodeupError::ApiError(
            "获取用户信息功能暂未实现".to_string(),
        ))
    }
}
