//! Jira 工作历史记录仓储实现
//!
//! 实现 `JiraWorkHistoryRepository` trait，通过内部服务层协调。

use std::sync::Arc;

use domain::{DeleteHistoryResult, JiraError, JiraWorkHistoryRepository, WorkHistoryEntry};

use crate::jira::history::services::WorkHistoryService;

/// Jira 工作历史记录仓储实现
pub struct JiraWorkHistoryRepositoryImpl {
    work_history_service: Arc<dyn WorkHistoryService>,
}

impl JiraWorkHistoryRepositoryImpl {
    /// 创建新的工作历史记录仓储实例
    pub fn new(work_history_service: Arc<dyn WorkHistoryService>) -> Self {
        Self {
            work_history_service,
        }
    }
}

impl JiraWorkHistoryRepository for JiraWorkHistoryRepositoryImpl {
    fn read_work_history(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<Option<String>, JiraError> {
        self.work_history_service.read_work_history(pull_request_id, repository)
    }

    fn read_work_history_entry(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<Option<WorkHistoryEntry>, JiraError> {
        self.work_history_service.read_work_history_entry(pull_request_id, repository)
    }

    fn find_pr_id_by_branch(
        &self,
        branch_name: &str,
        repository: &str,
    ) -> Result<Option<String>, JiraError> {
        self.work_history_service.find_pr_id_by_branch(branch_name, repository)
    }

    fn write_work_history(
        &self,
        jira_ticket: &str,
        pull_request_id: &str,
        pull_request_url: Option<&str>,
        repository: &str,
        branch: Option<&str>,
    ) -> Result<(), JiraError> {
        self.work_history_service.write_work_history(
            jira_ticket,
            pull_request_id,
            pull_request_url,
            repository,
            branch,
        )
    }

    fn update_work_history_merged(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<(), JiraError> {
        self.work_history_service
            .update_work_history_merged(pull_request_id, repository)
    }

    fn delete_work_history_entry(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<DeleteHistoryResult, JiraError> {
        self.work_history_service.delete_work_history_entry(pull_request_id, repository)
    }

    fn find_prs_by_jira_ticket(
        &self,
        jira_ticket: &str,
    ) -> Result<Vec<WorkHistoryEntry>, JiraError> {
        self.work_history_service.find_prs_by_jira_ticket(jira_ticket)
    }

    fn find_branches_by_jira_ticket(&self, jira_ticket: &str) -> Result<Vec<String>, JiraError> {
        self.work_history_service.find_branches_by_jira_ticket(jira_ticket)
    }
}
