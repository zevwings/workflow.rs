//! Pull Request 查询服务
//!
//! 提供 Pull Request 查询相关的业务逻辑实现

use std::{fmt::Write, sync::Arc};

use domain::CNBError;
use toolkit::log_debug;

use crate::cnb::client::CNBClient;
use crate::cnb::services::ServiceContext;
use crate::cnb::types::{CNBUserInfo, PullRequestInfo};

/// Pull Request 查询服务接口
pub trait PullRequestQueryService: Send + Sync {
    /// 获取 PR 信息
    fn get_pull_request_info(&self, pull_request_id: &str) -> Result<String, CNBError>;

    /// 获取 PR URL
    fn get_pull_request_url(&self, pull_request_id: &str) -> Result<String, CNBError>;

    /// 获取 PR 标题
    fn get_pull_request_title(&self, pull_request_id: &str) -> Result<String, CNBError>;

    /// 获取 PR body 内容
    fn get_pull_request_body(&self, pull_request_id: &str) -> Result<Option<String>, CNBError>;

    /// 获取 PR 状态
    fn get_pull_request_status(
        &self,
        pull_request_id: &str,
    ) -> Result<(String, bool, Option<String>), CNBError>;

    /// 获取 PR 列表
    fn get_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PullRequestInfo>, CNBError>;

    /// 获取当前分支的 PR ID
    fn get_current_branch_pull_request(
        &self,
        current_branch: &str,
    ) -> Result<Option<String>, CNBError>;

    /// 获取 PR 信息（内部方法）
    fn fetch_pr_info(&self, pr_number: &str) -> Result<PullRequestInfo, CNBError>;

    /// 获取 CNB 用户信息
    fn get_user_info(&self) -> Result<CNBUserInfo, CNBError>;

    /// 验证仓库是否存在并可访问
    ///
    /// 通过获取用户的仓库列表来验证当前项目路径是否有效
    fn verify_repository_access(&self) -> Result<bool, CNBError>;
}

/// Pull Request 查询服务实现
pub struct PullRequestQueryServiceImpl {
    client: Arc<dyn CNBClient>,
    context: Arc<dyn ServiceContext>,
}

impl PullRequestQueryServiceImpl {
    pub fn new(client: Arc<dyn CNBClient>, context: Arc<dyn ServiceContext>) -> Self {
        Self { client, context }
    }
}

impl PullRequestQueryService for PullRequestQueryServiceImpl {
    fn get_pull_request_info(&self, pull_request_id: &str) -> Result<String, CNBError> {
        let pr_number = self.context.parse_pr_number(pull_request_id)?;
        let pr = self.fetch_pr_info(&pr_number)?;

        let mut info = String::new();
        writeln!(info, "Title: {}", pr.title)
            .map_err(|e| CNBError::ApiError(format!("Failed to write info: {}", e)))?;
        if let Some(body) = pr.body {
            writeln!(info, "Description: {}", body)
                .map_err(|e| CNBError::ApiError(format!("Failed to write info: {}", e)))?;
        }
        writeln!(info, "State: {}", pr.state)
            .map_err(|e| CNBError::ApiError(format!("Failed to write info: {}", e)))?;
        writeln!(info, "Source Branch: {}", pr.head.ref_name)
            .map_err(|e| CNBError::ApiError(format!("Failed to write info: {}", e)))?;
        writeln!(info, "Target Branch: {}", pr.base.ref_name)
            .map_err(|e| CNBError::ApiError(format!("Failed to write info: {}", e)))?;
        if let Some(url) = pr.html_url {
            writeln!(info, "URL: {}", url)
                .map_err(|e| CNBError::ApiError(format!("Failed to write info: {}", e)))?;
        }

        Ok(info)
    }

    fn get_pull_request_url(&self, pull_request_id: &str) -> Result<String, CNBError> {
        let pr_number = self.context.parse_pr_number(pull_request_id)?;
        let pr = self.fetch_pr_info(&pr_number)?;
        pr.html_url
            .ok_or_else(|| CNBError::ApiError("PR URL not available".to_string()))
    }

    fn get_pull_request_title(&self, pull_request_id: &str) -> Result<String, CNBError> {
        let pr_number = self.context.parse_pr_number(pull_request_id)?;
        let pr = self.fetch_pr_info(&pr_number)?;
        Ok(pr.title)
    }

    fn get_pull_request_body(&self, pull_request_id: &str) -> Result<Option<String>, CNBError> {
        let pr_number = self.context.parse_pr_number(pull_request_id)?;
        let pr = self.fetch_pr_info(&pr_number)?;
        Ok(pr.body)
    }

    fn get_pull_request_status(
        &self,
        pull_request_id: &str,
    ) -> Result<(String, bool, Option<String>), CNBError> {
        let pr_number = self.context.parse_pr_number(pull_request_id)?;
        let pr = self.fetch_pr_info(&pr_number)?;
        Ok((pr.state, pr.merged, pr.merged_at))
    }

    fn get_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PullRequestInfo>, CNBError> {
        let project_path = self.context.project_path()?;
        let encoded_path = urlencoding::encode(&project_path);
        let mut url = format!("/repos/{}/pulls", encoded_path);

        // 添加查询参数
        let mut query_params = Vec::new();
        if let Some(state) = state {
            query_params.push(format!("state={}", state));
        }
        if let Some(limit) = limit {
            query_params.push(format!("per_page={}", limit));
        }

        if !query_params.is_empty() {
            url.push('?');
            url.push_str(&query_params.join("&"));
        }

        log_debug!("Fetching PR list from: {}", url);

        let response = self.client.get(&url)?;
        let prs: Vec<PullRequestInfo> = response.json()?;

        Ok(prs)
    }

    fn get_current_branch_pull_request(
        &self,
        current_branch: &str,
    ) -> Result<Option<String>, CNBError> {
        let project_path = self.context.project_path()?;
        let encoded_path = urlencoding::encode(&project_path);
        let url = format!(
            "/repos/{}/pulls?state=open&head={}",
            encoded_path, current_branch
        );

        log_debug!("Fetching PR for branch: {}", current_branch);

        let response = match self.client.get(&url) {
            Ok(resp) => resp,
            Err(CNBError::NotFound(_)) => {
                // 如果没有找到 PR，返回 None
                log_debug!("No PR found for branch: {}", current_branch);
                return Ok(None);
            }
            Err(e) => return Err(e),
        };

        // 尝试解析为 PR 列表
        match response.json::<Vec<PullRequestInfo>>() {
            Ok(prs) => Ok(prs.first().map(|pr| pr.number.clone())),
            Err(_) => {
                // 如果解析失败，可能是空响应或错误响应，返回 None
                log_debug!("Failed to parse PR list, assuming no PR exists");
                Ok(None)
            }
        }
    }

    fn fetch_pr_info(&self, pr_number: &str) -> Result<PullRequestInfo, CNBError> {
        let project_path = self.context.project_path()?;
        let encoded_path = urlencoding::encode(&project_path);
        let url = format!("/repos/{}/pulls/{}", encoded_path, pr_number);

        log_debug!("Fetching PR info from: {}", url);

        let response = self.client.get(&url)?;
        let pr: PullRequestInfo = response.json()?;

        Ok(pr)
    }

    fn get_user_info(&self) -> Result<CNBUserInfo, CNBError> {
        let url = "/user";

        log_debug!("Fetching user info from: {}", url);

        let response = self.client.get(url)?;
        let user: CNBUserInfo = response.json()?;

        Ok(user)
    }

    fn verify_repository_access(&self) -> Result<bool, CNBError> {
        let project_path = self.context.project_path()?;

        log_debug!("Verifying repository access for: {}", project_path);
        toolkit::log_info!("Checking if repository exists: {}", project_path);

        // 获取用户的所有仓库列表
        let url = "/user/repos";
        let response = self.client.get(url)?;

        // 解析仓库列表
        #[derive(serde::Deserialize)]
        struct RepoListItem {
            path: String,
            name: String,
        }

        let repos: Vec<RepoListItem> = response.json()?;

        // 检查项目路径是否在列表中
        let found = repos.iter().any(|repo| repo.path == project_path);

        if found {
            toolkit::log_info!("✓ Repository found in user's repository list");
            log_debug!("Repository verified: {}", project_path);
        } else {
            toolkit::log_error!("✗ Repository NOT found in user's repository list");
            toolkit::log_info!("Available repositories:");
            for repo in repos.iter().take(10) {
                toolkit::log_info!("  - {} ({})", repo.name, repo.path);
            }
            if repos.len() > 10 {
                toolkit::log_info!("  ... and {} more", repos.len() - 10);
            }
            toolkit::log_info!("Expected repository path: {}", project_path);
            toolkit::log_info!("💡 Tip: Check your git remote URL and CNB repository settings");
        }

        Ok(found)
    }
}
