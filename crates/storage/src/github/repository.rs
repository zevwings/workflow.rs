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
    types::PullRequestInfo as GitHubPullRequestInfo,
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
    fn convert_pr_info(pr_info: GitHubPullRequestInfo) -> PullRequestInfo {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::services::{
        PullRequestDiffService, PullRequestMutationService, PullRequestQueryService,
        PullRequestReviewService,
    };
    use crate::github::types::{GitHubUserInfo, PullRequestBranch, PullRequestInfo};
    use std::sync::{Arc, Mutex};

    struct MockQueryService {
        pr_info: PullRequestInfo,
        pr_id: Mutex<Option<String>>,
    }

    impl PullRequestQueryService for MockQueryService {
        fn get_pull_request_info(&self, pull_request_id: &str) -> Result<String, GitHubError> {
            *self.pr_id.lock().unwrap() = Some(pull_request_id.to_string());
            Ok("info".to_string())
        }

        fn get_pull_request_url(&self, _pull_request_id: &str) -> Result<String, GitHubError> {
            Ok(self.pr_info.html_url.clone())
        }

        fn get_pull_request_title(&self, _pull_request_id: &str) -> Result<String, GitHubError> {
            Ok(self.pr_info.title.clone())
        }

        fn get_pull_request_body(
            &self,
            _pull_request_id: &str,
        ) -> Result<Option<String>, GitHubError> {
            Ok(self.pr_info.body.clone())
        }

        fn get_pull_request_status(
            &self,
            _pull_request_id: &str,
        ) -> Result<(String, bool, Option<String>), GitHubError> {
            Ok((
                self.pr_info.state.clone(),
                self.pr_info.merged,
                self.pr_info.merged_at.clone(),
            ))
        }

        fn get_pull_requests(
            &self,
            _state: Option<&str>,
            _limit: Option<usize>,
        ) -> Result<Vec<PullRequestInfo>, GitHubError> {
            Ok(vec![self.pr_info.clone()])
        }

        fn get_current_branch_pull_request(
            &self,
            _current_branch: &str,
        ) -> Result<Option<String>, GitHubError> {
            Ok(Some(self.pr_info.number.to_string()))
        }

        fn fetch_pr_info(&self, _pr_number: u64) -> Result<PullRequestInfo, GitHubError> {
            Ok(self.pr_info.clone())
        }

        fn get_user_info(&self) -> Result<GitHubUserInfo, GitHubError> {
            Ok(GitHubUserInfo {
                login: "user".to_string(),
                name: Some("User".to_string()),
                email: Some("user@example.com".to_string()),
            })
        }
    }

    struct MockMutationService {
        pr_url: String,
    }

    impl PullRequestMutationService for MockMutationService {
        fn create_pull_request(
            &self,
            _title: &str,
            _body: &str,
            _source_branch: &str,
            _target_branch: &str,
        ) -> Result<String, GitHubError> {
            Ok(self.pr_url.clone())
        }

        fn merge_pull_request(
            &self,
            _pull_request_id: &str,
            _force: bool,
        ) -> Result<(), GitHubError> {
            Ok(())
        }

        fn close_pull_request(&self, _pull_request_id: &str) -> Result<(), GitHubError> {
            Ok(())
        }

        fn update_pr_base(
            &self,
            _pull_request_id: &str,
            _new_base: &str,
        ) -> Result<(), GitHubError> {
            Ok(())
        }

        fn update_pull_request(
            &self,
            _pull_request_id: &str,
            _title: Option<&str>,
            _body: Option<&str>,
        ) -> Result<(), GitHubError> {
            Ok(())
        }
    }

    struct MockReviewService;

    impl PullRequestReviewService for MockReviewService {
        fn add_comment(&self, _pull_request_id: &str, _comment: &str) -> Result<(), GitHubError> {
            Ok(())
        }

        fn approve(&self, _pull_request_id: &str) -> Result<(), GitHubError> {
            Ok(())
        }
    }

    struct MockDiffService;

    impl PullRequestDiffService for MockDiffService {
        fn get_pull_request_diff(&self, _pull_request_id: &str) -> Result<String, GitHubError> {
            Ok("diff".to_string())
        }
    }

    fn build_repo(pr_url: &str) -> GitHubRepositoryImpl {
        let pr_info = PullRequestInfo {
            number: 123,
            title: "title".to_string(),
            body: Some("body".to_string()),
            state: "open".to_string(),
            merged: false,
            merged_at: None,
            html_url: pr_url.to_string(),
            head: PullRequestBranch {
                ref_name: "feature".to_string(),
            },
            base: PullRequestBranch {
                ref_name: "main".to_string(),
            },
            user: None,
        };

        GitHubRepositoryImpl::new(
            Arc::new(MockQueryService {
                pr_info,
                pr_id: Mutex::new(None),
            }),
            Arc::new(MockMutationService {
                pr_url: pr_url.to_string(),
            }),
            Arc::new(MockReviewService),
            Arc::new(MockDiffService),
        )
    }

    #[test]
    fn test_create_pull_request_validation() {
        let repo = build_repo("https://github.com/owner/repo/pull/123");

        let err = repo.create_pull_request("", "body", "feature", "main").unwrap_err();
        assert!(err.to_string().contains("title"));

        let err = repo.create_pull_request("title", "body", "", "main").unwrap_err();
        assert!(err.to_string().contains("Source branch"));

        let err = repo.create_pull_request("title", "body", "feature", "").unwrap_err();
        assert!(err.to_string().contains("Target branch"));
    }

    #[test]
    fn test_create_pull_request_extracts_id() {
        let repo = build_repo("https://github.com/owner/repo/pull/456");
        let pr_id = repo.create_pull_request("title", "body", "feature", "main").unwrap();
        assert_eq!(pr_id, "456");
    }

    #[test]
    fn test_get_pull_request_converts_fields() {
        let repo = build_repo("https://github.com/owner/repo/pull/123");
        let pr = repo.get_pull_request("123").unwrap();
        assert_eq!(pr.id, "123");
        assert_eq!(pr.title, "title");
        assert_eq!(pr.source_branch, "feature");
        assert_eq!(pr.target_branch, "main");
    }

    #[test]
    fn test_add_comment_rejects_empty() {
        let repo = build_repo("https://github.com/owner/repo/pull/123");
        let result = repo.add_comment("123", "");
        assert!(result.is_err());
    }

    #[test]
    fn test_update_pr_base_rejects_empty() {
        let repo = build_repo("https://github.com/owner/repo/pull/123");
        let result = repo.update_pr_base("123", "");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_current_branch_pull_request_rejects_empty() {
        let repo = build_repo("https://github.com/owner/repo/pull/123");
        let result = repo.get_current_branch_pull_request("");
        assert!(result.is_err());
    }
}
