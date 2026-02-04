//! Jira 仓储实现
//!
//! 实现 `JiraRepository` trait，通过内部服务层协调各个组件。
//! Repository 层作为极薄的委托层，所有业务逻辑都在 services 层。

use std::sync::Arc;

use crate::jira::services::{IssueService, StatusService, UserService};
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
        Err(JiraError::Other(
            "Attachment download functionality is currently disabled. See jira/services/attachment/mod.rs for details.".to_string()
        ))
    }

    fn clean_attachments(&self, _jira_id: Option<&str>) -> Result<(), JiraError> {
        Err(JiraError::Other(
            "Attachment cleanup functionality is currently disabled. See jira/services/attachment/mod.rs for details.".to_string()
        ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jira::services::{IssueService, StatusService, UserService};
    use crate::jira::types::{JiraAttachment as StorageAttachment, JiraIssue as StorageIssue};
    use std::sync::Mutex;

    struct MockIssueService {
        issue: JiraIssue,
        attachments: Vec<JiraAttachment>,
        update_called: Mutex<bool>,
        comment_called: Mutex<bool>,
    }

    impl IssueService for MockIssueService {
        fn get_issue_info(&self, _issue_id: &str) -> Result<JiraIssue, JiraError> {
            Ok(self.issue.clone())
        }

        fn update_issue_status(&self, _issue_id: &str, _status: &str) -> Result<(), JiraError> {
            *self.update_called.lock().unwrap() = true;
            Ok(())
        }

        fn add_comment(&self, _issue_id: &str, _comment: &str) -> Result<(), JiraError> {
            *self.comment_called.lock().unwrap() = true;
            Ok(())
        }

        fn get_attachments(&self, _issue_id: &str) -> Result<Vec<JiraAttachment>, JiraError> {
            Ok(self.attachments.clone())
        }

        fn fetch_issue_data(
            &self,
            _issue_id: &str,
        ) -> Result<(StorageIssue, Vec<StorageAttachment>, Option<String>), JiraError> {
            Err(JiraError::ApiError("not implemented".to_string()))
        }
    }

    struct MockStatusService {
        statuses: Vec<String>,
        created: Option<String>,
        merged: Option<String>,
    }

    impl StatusService for MockStatusService {
        fn get_project_statuses(&self, _project: &str) -> Result<Vec<String>, JiraError> {
            Ok(self.statuses.clone())
        }

        fn write_status_config(&self, _config: &JiraStatusConfig) -> Result<(), JiraError> {
            Ok(())
        }

        fn read_pull_request_created_status(
            &self,
            _jira_ticket: &str,
        ) -> Result<Option<String>, JiraError> {
            Ok(self.created.clone())
        }

        fn read_pull_request_merged_status(
            &self,
            _jira_ticket: &str,
        ) -> Result<Option<String>, JiraError> {
            Ok(self.merged.clone())
        }
    }

    struct MockUserService {
        user: JiraUser,
    }

    impl UserService for MockUserService {
        fn get_user_info(&self) -> Result<JiraUser, JiraError> {
            Ok(self.user.clone())
        }
    }

    #[test]
    fn test_repository_delegates_calls() {
        let issue = JiraIssue {
            id: "100".to_string(),
            key: "PROJ-1".to_string(),
            summary: "Summary".to_string(),
            status: "Open".to_string(),
            assignee: None,
            description: None,
            attachments: vec![],
            comments: vec![],
            priority: None,
            created: None,
            updated: None,
            reporter: None,
            labels: vec![],
            components: vec![],
        };
        let attachments = vec![JiraAttachment {
            id: "a".to_string(),
            filename: "log.txt".to_string(),
            size: 1,
            url: "https://example.com/log.txt".to_string(),
        }];

        let issue_service = Arc::new(MockIssueService {
            issue,
            attachments,
            update_called: Mutex::new(false),
            comment_called: Mutex::new(false),
        });
        let status_service = Arc::new(MockStatusService {
            statuses: vec!["Open".to_string()],
            created: Some("In Review".to_string()),
            merged: Some("Done".to_string()),
        });
        let user_service = Arc::new(MockUserService {
            user: JiraUser {
                display_name: "User".to_string(),
                account_id: "123".to_string(),
            },
        });

        let repo = JiraRepositoryImpl::new(issue_service.clone(), status_service, user_service);

        let user = repo.get_user_info().unwrap();
        assert_eq!(user.display_name, "User");

        let issue = repo.get_issue_info("PROJ-1").unwrap();
        assert_eq!(issue.key, "PROJ-1");

        repo.update_issue_status("PROJ-1", "Done").unwrap();
        assert!(*issue_service.update_called.lock().unwrap());

        repo.add_comment("PROJ-1", "hello").unwrap();
        assert!(*issue_service.comment_called.lock().unwrap());

        let attachments = repo.get_attachments("PROJ-1").unwrap();
        assert_eq!(attachments.len(), 1);

        let created = repo.read_pull_request_created_status("PROJ-1").unwrap();
        assert_eq!(created, Some("In Review".to_string()));
    }
}
