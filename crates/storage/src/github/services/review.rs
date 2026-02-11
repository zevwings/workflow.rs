//! Pull Request 评论和审批服务
//!
//! 提供 Pull Request 评论和审批相关的业务逻辑实现

use std::sync::Arc;

use domain::GitHubError;

use crate::github::{
    client::GitHubClient,
    services::{PullRequestQueryService, ServiceContext},
};

const PR_APPROVE_EVENT: &str = "APPROVE";
const PR_APPROVE_EMOJI: &str = "👍";

/// Pull Request 评论和审批服务接口
pub trait PullRequestReviewService: Send + Sync {
    /// 添加评论到 Pull Request
    fn add_comment(&self, pull_request_id: &str, comment: &str) -> Result<(), GitHubError>;

    /// 批准 Pull Request
    fn approve(&self, pull_request_id: &str) -> Result<(), GitHubError>;
}

/// Pull Request 评论和审批服务实现
pub struct PullRequestReviewServiceImpl {
    client: Arc<dyn GitHubClient>,
    query_service: Arc<dyn PullRequestQueryService>,
    context: Arc<dyn ServiceContext>,
}

impl PullRequestReviewServiceImpl {
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
}

impl PullRequestReviewService for PullRequestReviewServiceImpl {
    fn add_comment(&self, pull_request_id: &str, comment: &str) -> Result<(), GitHubError> {
        #[derive(serde::Serialize)]
        struct CommentRequest {
            body: String,
        }

        let (owner, repo_name) = self.context.get_owner_and_repo()?;
        let pr_number = self.context.parse_pr_number(pull_request_id)?;

        let url = format!(
            "/repos/{}/{}/issues/{}/comments",
            owner, repo_name, pr_number
        );

        let request = CommentRequest {
            body: comment.to_string(),
        };

        let body = serde_json::to_value(&request)
            .map_err(|e| GitHubError::ApiError(format!("Failed to serialize request: {}", e)))?;
        self.client.post(&url, &body)?;

        Ok(())
    }

    fn approve(&self, pull_request_id: &str) -> Result<(), GitHubError> {
        #[derive(serde::Serialize)]
        struct ReviewRequest {
            event: String,
            body: String,
        }

        let (owner, repo_name) = self.context.get_owner_and_repo()?;
        let pr_number = self.context.parse_pr_number(pull_request_id)?;

        let pr_info = self.query_service.fetch_pr_info(pr_number)?;
        let current_user = self.query_service.get_user_info()?;

        if let Some(ref pr_user) = pr_info.user {
            if pr_user.login == current_user.login {
                return Err(GitHubError::ApiError(
                    "Cannot approve your own pull request. GitHub does not allow users to approve their own PRs.".to_string()
                ));
            }
        }

        let url = format!("/repos/{}/{}/pulls/{}/reviews", owner, repo_name, pr_number);

        let request = ReviewRequest {
            event: PR_APPROVE_EVENT.to_string(),
            body: PR_APPROVE_EMOJI.to_string(),
        };

        let body = serde_json::to_value(&request)
            .map_err(|e| GitHubError::ApiError(format!("Failed to serialize request: {}", e)))?;

        self.client.post(&url, &body)?;

        Ok(())
    }
}
