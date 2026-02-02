//! Pull Request 评论和审批服务
//!
//! 提供 Pull Request 评论和审批相关的业务逻辑实现

use std::sync::Arc;

use domain::CNBError;
use toolkit::log_debug;

use crate::cnb::client::CNBClient;
use crate::cnb::services::{PullRequestQueryService, ServiceContext};
use crate::cnb::types::{CreateComment, CreateReview};

const PR_APPROVE_EVENT: &str = "APPROVE";
const PR_APPROVE_EMOJI: &str = "👍";

/// Pull Request 评论和审批服务接口
pub trait PullRequestReviewService: Send + Sync {
    /// 添加评论到 Pull Request
    fn add_comment(&self, pull_request_id: &str, comment: &str) -> Result<(), CNBError>;

    /// 批准 Pull Request
    fn approve(&self, pull_request_id: &str) -> Result<(), CNBError>;
}

/// Pull Request 评论和审批服务实现
pub struct PullRequestReviewServiceImpl {
    client: Arc<dyn CNBClient>,
    query_service: Arc<dyn PullRequestQueryService>,
    context: Arc<dyn ServiceContext>,
}

impl PullRequestReviewServiceImpl {
    pub fn new(
        client: Arc<dyn CNBClient>,
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
    fn add_comment(&self, pull_request_id: &str, comment: &str) -> Result<(), CNBError> {
        let project_path = self.context.project_path()?;
        let encoded_path = urlencoding::encode(&project_path);
        let pr_number = self.context.parse_pr_number(pull_request_id)?;

        // CNB 使用 /repos/{project}/pulls/{number}/comments 端点
        let url = format!("/repos/{}/pulls/{}/comments", encoded_path, pr_number);

        let request = CreateComment {
            body: comment.to_string(),
        };

        log_debug!("Adding comment to PR: {}", pr_number);

        let body = serde_json::to_value(&request)
            .map_err(|e| CNBError::ApiError(format!("Failed to serialize request: {}", e)))?;
        self.client.post(&url, &body)?;

        Ok(())
    }

    fn approve(&self, pull_request_id: &str) -> Result<(), CNBError> {
        let project_path = self.context.project_path()?;
        let encoded_path = urlencoding::encode(&project_path);
        let pr_number = self.context.parse_pr_number(pull_request_id)?;

        // 检查是否是自己的 PR
        let pr_info = self.query_service.fetch_pr_info(&pr_number)?;
        let current_user = self.query_service.get_user_info()?;

        if let Some(ref author) = pr_info.author {
            if author.login == current_user.login {
                return Err(CNBError::ApiError(
                    "Cannot approve your own pull request. CNB does not allow users to approve their own PRs.".to_string()
                ));
            }
        }

        let url = format!("/repos/{}/pulls/{}/reviews", encoded_path, pr_number);

        let request = CreateReview {
            body: Some(PR_APPROVE_EMOJI.to_string()),
            event: PR_APPROVE_EVENT.to_string(),
            comments: None,
        };

        log_debug!("Approving PR: {}", pr_number);

        let body = serde_json::to_value(&request)
            .map_err(|e| CNBError::ApiError(format!("Failed to serialize request: {}", e)))?;
        self.client.post(&url, &body)?;

        Ok(())
    }
}
