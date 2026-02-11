//! GitHub 仓储实现
//!
//! 实现 `GitHubRepository` trait，通过内部服务层协调各个组件。
//! Repository 层作为极薄的委托层，所有业务逻辑都在 services 层。

use std::sync::Arc;

use domain::{GitHubError, GitHubRepository, GitHubUser, PullRequestInfo, PullRequestStatus};

use crate::github::{
    services::{
        PullRequestDiffService, PullRequestMutationService, PullRequestQueryService,
        PullRequestReviewService,
    },
    types::PullRequestInfo as GitHubPrInfo,
};

/// GitHub 仓储实现
///
/// 实现 `GitHubRepository` trait，通过依赖注入使用内部服务层。
pub struct GitHubRepositoryImpl {
    query_service: Arc<dyn PullRequestQueryService>,
    mutation_service: Arc<dyn PullRequestMutationService>,
    review_service: Arc<dyn PullRequestReviewService>,
    diff_service: Arc<dyn PullRequestDiffService>,
}

impl GitHubRepositoryImpl {
    pub fn new(
        query_service: Arc<dyn PullRequestQueryService>,
        mutation_service: Arc<dyn PullRequestMutationService>,
        review_service: Arc<dyn PullRequestReviewService>,
        diff_service: Arc<dyn PullRequestDiffService>,
    ) -> Self {
        Self {
            query_service,
            mutation_service,
            review_service,
            diff_service,
        }
    }

    /// 验证 PR ID 不为空
    fn validate_pr_id(&self, pr_id: &str) -> Result<(), GitHubError> {
        if pr_id.is_empty() {
            return Err(GitHubError::ApiError("PR ID cannot be empty".to_string()));
        }
        Ok(())
    }

    /// 将 GitHub API 的 PullRequestInfo 转换为 domain 的 PullRequestInfo
    fn convert_pr_info(pr_info: GitHubPrInfo) -> PullRequestInfo {
        let status = PullRequestStatus {
            state: pr_info.state,
            merged: pr_info.merged,
            merged_at: pr_info.merged_at,
        };

        PullRequestInfo {
            id: pr_info.number.to_string(),
            title: pr_info.title,
            body: pr_info.body.unwrap_or_default(),
            status,
            source_branch: pr_info.head.ref_name,
            target_branch: pr_info.base.ref_name,
        }
    }
}

impl GitHubRepository for GitHubRepositoryImpl {
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<String, GitHubError> {
        if title.is_empty() {
            return Err(GitHubError::ApiError(
                "PR title cannot be empty".to_string(),
            ));
        }
        if source_branch.is_empty() {
            return Err(GitHubError::ApiError(
                "Source branch cannot be empty".to_string(),
            ));
        }
        if target_branch.is_empty() {
            return Err(GitHubError::ApiError(
                "Target branch cannot be empty".to_string(),
            ));
        }

        let pr_url =
            self.mutation_service
                .create_pull_request(title, body, source_branch, target_branch)?;

        // 从 URL 中提取 PR ID（例如：https://github.com/owner/repo/pull/123 -> 123）
        let pr_id = pr_url
            .rsplit('/')
            .next()
            .ok_or_else(|| {
                GitHubError::ApiError(format!("Failed to extract PR ID from URL: {}", pr_url))
            })?
            .to_string();

        Ok(pr_id)
    }

    fn get_pull_request(&self, pr_id: &str) -> Result<PullRequestInfo, GitHubError> {
        self.validate_pr_id(pr_id)?;

        let pr_number = pr_id
            .parse::<u64>()
            .map_err(|e| GitHubError::ApiError(format!("Invalid PR ID '{}': {}", pr_id, e)))?;

        let pr_info = self.query_service.fetch_pr_info(pr_number)?;
        Ok(Self::convert_pr_info(pr_info))
    }

    fn merge_pull_request(&self, pr_id: &str, force: bool) -> Result<(), GitHubError> {
        self.validate_pr_id(pr_id)?;
        self.mutation_service.merge_pull_request(pr_id, force)
    }

    fn get_user_info(&self) -> Result<GitHubUser, GitHubError> {
        let user = self.query_service.get_user_info()?;

        Ok(GitHubUser {
            login: user.login,
            name: user.name,
            email: user.email,
        })
    }

    fn close_pull_request(&self, pr_id: &str) -> Result<(), GitHubError> {
        self.validate_pr_id(pr_id)?;
        self.mutation_service.close_pull_request(pr_id)
    }

    fn list_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PullRequestInfo>, GitHubError> {
        let prs = self.query_service.get_pull_requests(state, limit)?;
        Ok(prs.into_iter().map(Self::convert_pr_info).collect())
    }

    fn update_pull_request(
        &self,
        pr_id: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<(), GitHubError> {
        self.validate_pr_id(pr_id)?;
        self.mutation_service.update_pull_request(pr_id, title, body)
    }

    fn add_comment(&self, pr_id: &str, comment: &str) -> Result<(), GitHubError> {
        self.validate_pr_id(pr_id)?;
        if comment.is_empty() {
            return Err(GitHubError::ApiError("Comment cannot be empty".to_string()));
        }
        self.review_service.add_comment(pr_id, comment)
    }

    fn approve_pull_request(&self, pr_id: &str) -> Result<(), GitHubError> {
        self.validate_pr_id(pr_id)?;
        self.review_service.approve(pr_id)
    }

    fn get_pr_diff(&self, pr_id: &str) -> Result<String, GitHubError> {
        self.validate_pr_id(pr_id)?;
        self.diff_service.get_pull_request_diff(pr_id)
    }

    fn get_pull_request_info(&self, pr_id: &str) -> Result<String, GitHubError> {
        self.validate_pr_id(pr_id)?;
        self.query_service.get_pull_request_info(pr_id)
    }

    fn get_pull_request_url(&self, pr_id: &str) -> Result<String, GitHubError> {
        self.validate_pr_id(pr_id)?;
        self.query_service.get_pull_request_url(pr_id)
    }

    fn get_pull_request_title(&self, pr_id: &str) -> Result<String, GitHubError> {
        self.validate_pr_id(pr_id)?;
        self.query_service.get_pull_request_title(pr_id)
    }

    fn get_pull_request_body(&self, pr_id: &str) -> Result<Option<String>, GitHubError> {
        self.validate_pr_id(pr_id)?;
        self.query_service.get_pull_request_body(pr_id)
    }

    fn get_pull_request_status(
        &self,
        pr_id: &str,
    ) -> Result<(String, bool, Option<String>), GitHubError> {
        self.validate_pr_id(pr_id)?;
        self.query_service.get_pull_request_status(pr_id)
    }

    fn update_pr_base(&self, pr_id: &str, new_base: &str) -> Result<(), GitHubError> {
        self.validate_pr_id(pr_id)?;
        if new_base.is_empty() {
            return Err(GitHubError::ApiError(
                "New base branch cannot be empty".to_string(),
            ));
        }
        self.mutation_service.update_pr_base(pr_id, new_base)
    }

    fn get_current_branch_pull_request(
        &self,
        current_branch: &str,
    ) -> Result<Option<String>, GitHubError> {
        if current_branch.is_empty() {
            return Err(GitHubError::ApiError(
                "Current branch cannot be empty".to_string(),
            ));
        }
        self.query_service.get_current_branch_pull_request(current_branch)
    }
}
