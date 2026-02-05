//! Jira 仓储实现
//!
//! 实现 `JiraRepository` trait，通过内部服务层协调各个组件。
//! Repository 层作为极薄的委托层，所有业务逻辑都在 services 层。

use std::sync::Arc;

use crate::jira::api::services::{IssueService, StatusService, UserService};
use domain::{
    AttachmentDownloadResult, JiraAttachment, JiraError, JiraIssue, JiraRepository,
    JiraStatusConfig, JiraUser,
};

/// Jira 仓储实现
///
/// 实现 `JiraRepository` trait，通过依赖注入使用内部服务层。
pub struct JiraRepositoryImpl {
    issue_service: Arc<dyn IssueService>,
    status_service: Arc<dyn StatusService>,
    user_service: Arc<dyn UserService>,
}

impl JiraRepositoryImpl {
    pub fn new(
        issue_service: Arc<dyn IssueService>,
        status_service: Arc<dyn StatusService>,
        user_service: Arc<dyn UserService>,
    ) -> Self {
        Self {
            issue_service,
            status_service,
            user_service,
        }
    }
}

impl JiraRepository for JiraRepositoryImpl {
    fn get_user_info(&self) -> Result<JiraUser, JiraError> {
        self.user_service.get_user_info()
    }
    fn get_issue_info(&self, issue_id: &str) -> Result<JiraIssue, JiraError> {
        self.issue_service.get_issue_info(issue_id)
    }

    fn update_issue_status(&self, issue_id: &str, status: &str) -> Result<(), JiraError> {
        self.issue_service.update_issue_status(issue_id, status)
    }

    fn add_comment(&self, issue_id: &str, comment: &str) -> Result<(), JiraError> {
        self.issue_service.add_comment(issue_id, comment)
    }

    fn get_attachments(&self, issue_id: &str) -> Result<Vec<JiraAttachment>, JiraError> {
        self.issue_service.get_attachments(issue_id)
    }

    fn download_attachments(
        &self,
        _issue_id: &str,
        _base_dir: &std::path::Path,
    ) -> Result<AttachmentDownloadResult, JiraError> {
        unimplemented!()
    }

    fn clean_attachments(&self, _jira_id: Option<&str>) -> Result<(), JiraError> {
        unimplemented!()
    }

    fn get_project_statuses(&self, project: &str) -> Result<Vec<String>, JiraError> {
        self.status_service.get_project_statuses(project)
    }

    fn write_status_config(&self, config: &JiraStatusConfig) -> Result<(), JiraError> {
        self.status_service.write_status_config(config)
    }

    fn read_pull_request_created_status(
        &self,
        jira_ticket: &str,
    ) -> Result<Option<String>, JiraError> {
        self.status_service.read_pull_request_created_status(jira_ticket)
    }

    fn read_pull_request_merged_status(
        &self,
        jira_ticket: &str,
    ) -> Result<Option<String>, JiraError> {
        self.status_service.read_pull_request_merged_status(jira_ticket)
    }
}
