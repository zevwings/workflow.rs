use std::sync::Arc;

use domain::{
    LLMError, LLMRepository, PrContent, PullRequestContent, PullRequestReword, PullRequestSummary,
};

use crate::llm::services::LLMService;

/// LLM 服务实现
///
/// 实现 `LLMRepository` trait，提供 LLM API 操作。
/// 作为适配器层，将 domain 接口委托给 Service 实现。
pub struct LLMRepositoryImpl {
    service: Arc<dyn LLMService>,
}

impl LLMRepositoryImpl {
    pub fn new(service: Arc<dyn LLMService>) -> Self {
        Self { service }
    }
}

impl LLMRepository for LLMRepositoryImpl {
    fn verify_config(&self) -> Result<String, LLMError> {
        self.service.verify_config()
    }

    fn generate_branch_name(
        &self,
        title: Option<&str>,
        exists_branches: Option<Vec<String>>,
    ) -> Result<String, LLMError> {
        self.service.generate_branch_name(title, exists_branches)
    }

    fn generate_pr_content(
        &self,
        branch_name: &str,
        commits: &[String],
    ) -> Result<PrContent, LLMError> {
        self.service.generate_pr_content(branch_name, commits)
    }

    fn generate_commit_message(&self, changes: &str) -> Result<String, LLMError> {
        self.service.generate_commit_message(changes)
    }

    fn translate_to_english(&self, text: &str) -> Result<String, LLMError> {
        self.service.translate_to_english(text)
    }

    fn create_pr_content(
        &self,
        commit_title: &str,
        exists_branches: Option<Vec<String>>,
        git_diff: Option<String>,
    ) -> Result<PullRequestContent, LLMError> {
        self.service
            .create_pr_content(commit_title, exists_branches, git_diff)
    }

    fn reword_pr(
        &self,
        pr_diff: &str,
        current_title: Option<&str>,
    ) -> Result<PullRequestReword, LLMError> {
        self.service.reword_pr(pr_diff, current_title)
    }

    fn summarize_pr(&self, pr_title: &str, pr_diff: &str) -> Result<PullRequestSummary, LLMError> {
        self.service.summarize_pr(pr_title, pr_diff)
    }

    fn summarize_file_change(&self, file_path: &str, file_diff: &str) -> Result<String, LLMError> {
        self.service.summarize_file_change(file_path, file_diff)
    }
}
