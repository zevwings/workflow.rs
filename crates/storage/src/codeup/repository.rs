//! Codeup 仓储实现

use std::sync::Arc;

use domain::{CodeupError, CodeupRepository, CodeupUser, PullRequestInfo as DomainPullRequestInfo};

use crate::codeup::services::{
    PullRequestMutationService, PullRequestQueryService, ServiceContext,
};

/// Codeup 仓储实现
pub struct CodeupRepositoryImpl {
    mutation_service: Arc<dyn PullRequestMutationService>,
    query_service: Arc<dyn PullRequestQueryService>,
    #[allow(dead_code)]
    context: Arc<dyn ServiceContext>,
}

impl CodeupRepositoryImpl {
    pub fn new(
        mutation_service: Arc<dyn PullRequestMutationService>,
        query_service: Arc<dyn PullRequestQueryService>,
        context: Arc<dyn ServiceContext>,
    ) -> Self {
        Self {
            mutation_service,
            query_service,
            context,
        }
    }

    /// 验证 PR ID
    fn validate_pr_id(&self, pr_id: &str) -> Result<(), CodeupError> {
        if pr_id.is_empty() {
            return Err(CodeupError::ApiError("PR ID 不能为空".to_string()));
        }
        Ok(())
    }
}

impl CodeupRepository for CodeupRepositoryImpl {
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<String, CodeupError> {
        if title.is_empty() {
            return Err(CodeupError::ApiError("PR 标题不能为空".to_string()));
        }
        if source_branch.is_empty() {
            return Err(CodeupError::ApiError("源分支不能为空".to_string()));
        }
        if target_branch.is_empty() {
            return Err(CodeupError::ApiError("目标分支不能为空".to_string()));
        }

        let pr_url =
            self.mutation_service
                .create_pull_request(title, body, source_branch, target_branch)?;

        // 从 URL 中提取 PR ID
        let pr_id = pr_url
            .rsplit('/')
            .next()
            .ok_or_else(|| CodeupError::ApiError(format!("无法从 URL 提取 PR ID: {}", pr_url)))?
            .to_string();

        Ok(pr_id)
    }

    fn get_pull_request(&self, pr_id: &str) -> Result<DomainPullRequestInfo, CodeupError> {
        self.validate_pr_id(pr_id)?;
        // 需要将 Codeup 的 PullRequestInfo 转换为 domain 的 PullRequestInfo
        // 这里简化处理，实际应该做类型转换
        Err(CodeupError::ApiError("需要实现类型转换".to_string()))
    }

    fn merge_pull_request(&self, pr_id: &str, force: bool) -> Result<(), CodeupError> {
        self.validate_pr_id(pr_id)?;
        self.mutation_service.merge_pull_request(pr_id, force)
    }

    fn get_user_info(&self) -> Result<CodeupUser, CodeupError> {
        self.query_service.get_user_info()
    }

    fn close_pull_request(&self, pr_id: &str) -> Result<(), CodeupError> {
        self.validate_pr_id(pr_id)?;
        self.mutation_service.close_pull_request(pr_id)
    }

    fn list_pull_requests(
        &self,
        _state: Option<&str>,
        _limit: Option<usize>,
    ) -> Result<Vec<DomainPullRequestInfo>, CodeupError> {
        // 需要类型转换
        Err(CodeupError::ApiError("需要实现类型转换".to_string()))
    }

    fn update_pull_request(
        &self,
        pr_id: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<(), CodeupError> {
        self.validate_pr_id(pr_id)?;
        self.mutation_service.update_pull_request(pr_id, title, body)
    }

    fn add_comment(&self, pr_id: &str, comment: &str) -> Result<(), CodeupError> {
        self.validate_pr_id(pr_id)?;
        if comment.is_empty() {
            return Err(CodeupError::ApiError("评论内容不能为空".to_string()));
        }
        // 需要实现评论服务
        Err(CodeupError::ApiError("评论功能暂未实现".to_string()))
    }

    fn approve_pull_request(&self, pr_id: &str) -> Result<(), CodeupError> {
        self.validate_pr_id(pr_id)?;
        // 需要实现批准服务
        Err(CodeupError::ApiError("批准功能暂未实现".to_string()))
    }

    fn get_pr_diff(&self, pr_id: &str) -> Result<String, CodeupError> {
        self.validate_pr_id(pr_id)?;
        // 需要实现 diff 获取服务
        Err(CodeupError::ApiError("获取 diff 功能暂未实现".to_string()))
    }

    fn get_pull_request_info(&self, pr_id: &str) -> Result<String, CodeupError> {
        self.validate_pr_id(pr_id)?;
        self.query_service.get_pull_request_info(pr_id)
    }

    fn get_pull_request_url(&self, pr_id: &str) -> Result<String, CodeupError> {
        self.validate_pr_id(pr_id)?;
        self.query_service.get_pull_request_url(pr_id)
    }

    fn get_pull_request_title(&self, pr_id: &str) -> Result<String, CodeupError> {
        self.validate_pr_id(pr_id)?;
        self.query_service.get_pull_request_title(pr_id)
    }

    fn get_pull_request_body(&self, pr_id: &str) -> Result<Option<String>, CodeupError> {
        self.validate_pr_id(pr_id)?;
        self.query_service.get_pull_request_body(pr_id)
    }

    fn get_pull_request_status(
        &self,
        pr_id: &str,
    ) -> Result<(String, bool, Option<String>), CodeupError> {
        self.validate_pr_id(pr_id)?;
        self.query_service.get_pull_request_status(pr_id)
    }

    fn update_pr_base(&self, pr_id: &str, new_base: &str) -> Result<(), CodeupError> {
        self.validate_pr_id(pr_id)?;
        if new_base.is_empty() {
            return Err(CodeupError::ApiError("新 base 分支不能为空".to_string()));
        }
        // 需要通过更新 PR 来实现
        self.mutation_service.update_pull_request(pr_id, None, None)
    }

    fn get_current_branch_pull_request(
        &self,
        current_branch: &str,
    ) -> Result<Option<String>, CodeupError> {
        if current_branch.is_empty() {
            return Err(CodeupError::ApiError("当前分支不能为空".to_string()));
        }
        self.query_service.get_current_branch_pull_request(current_branch)
    }
}
