//! Issue 数据获取服务
//!
//! 提供统一的 Issue 数据获取和操作功能，包括：
//! - 从 API 获取 Issue 数据
//! - 解析附件（从 API 和描述中）
//! - 合并和去重附件
//! - Domain 类型转换
//! - Issue 操作（状态更新、添加评论）

use std::sync::Arc;

use serde::Serialize;

use domain::{JiraAttachment, JiraComment, JiraError, JiraIssue, JiraUser};

use crate::jira::client::core::JiraClient;
use crate::jira::client::types::JiraResponseSerializable;
use crate::jira::types::{
    JiraAttachment as StorageJiraAttachment, JiraIssue as StorageJiraIssue, JiraTransition,
};

// 文件私有请求类型

#[derive(Serialize)]
struct TransitionRequest {
    transition: TransitionRef,
}

#[derive(Serialize)]
struct TransitionRef {
    id: String,
}

#[derive(Serialize)]
struct CommentRequest {
    body: String,
}

pub trait IssueService: Send + Sync {
    fn get_issue_info(&self, issue_id: &str) -> Result<JiraIssue, JiraError>;
    fn update_issue_status(&self, issue_id: &str, status: &str) -> Result<(), JiraError>;
    fn add_comment(&self, issue_id: &str, comment: &str) -> Result<(), JiraError>;
    fn get_attachments(&self, issue_id: &str) -> Result<Vec<JiraAttachment>, JiraError>;
    fn fetch_issue_data(
        &self,
        issue_id: &str,
    ) -> Result<(StorageJiraIssue, Vec<StorageJiraAttachment>, Option<String>), JiraError>;
}

/// Issue 数据获取服务
///
/// 提供统一的 Issue 数据获取功能，避免重复的 API 调用。
pub struct IssueServiceImpl {
    jira_client: Arc<dyn JiraClient>,
}

impl IssueService for IssueServiceImpl {
    fn get_issue_info(&self, issue_id: &str) -> Result<JiraIssue, JiraError> {
        let (issue, attachments, _) = self
            .fetch_issue_data(issue_id)
            .map_err(|e| JiraError::ApiError(format!("Failed to get issue {}: {}", issue_id, e)))?;

        // DTO → Domain 映射
        Ok(JiraIssue {
            id: issue.id,
            key: issue.key,
            summary: issue.fields.summary,
            status: issue.fields.status.name,
            assignee: issue.fields.assignee.as_ref().map(|u| u.display_name.clone()),
            description: issue.fields.description.clone(),
            attachments: attachments
                .iter()
                .map(|a| JiraAttachment {
                    id: a.filename.clone(), // Jira 附件可能没有单独的 ID，使用 filename 作为标识
                    filename: a.filename.clone(),
                    size: a.size.unwrap_or(0),
                    url: a.content_url.clone(),
                })
                .collect(),
            comments: issue
                .fields
                .comment
                .as_ref()
                .map(|c| {
                    c.comments
                        .iter()
                        .map(|comment| JiraComment {
                            id: comment.id.clone(),
                            body: comment.body.clone(),
                            created: comment.created.clone(),
                            author: comment.author.as_ref().map(|u| JiraUser {
                                display_name: u.display_name.clone(),
                                account_id: u.account_id.clone(),
                            }),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            priority: issue.fields.priority.as_ref().map(|p| p.name.clone()),
            created: issue.fields.created.clone(),
            updated: issue.fields.updated.clone(),
            reporter: issue.fields.reporter.as_ref().map(|u| JiraUser {
                display_name: u.display_name.clone(),
                account_id: u.account_id.clone(),
            }),
            labels: issue.fields.labels.unwrap_or_default(),
            components: issue
                .fields
                .components
                .unwrap_or_default()
                .into_iter()
                .map(|c| c.name)
                .collect(),
        })
    }

    fn update_issue_status(&self, issue_id: &str, status: &str) -> Result<(), JiraError> {
        // 1. 获取可用的 transitions
        let path = format!("issue/{}/transitions", issue_id);
        let response = self.jira_client.get(&path, None).map_err(|e| {
            JiraError::ApiError(format!(
                "Failed to get transitions for issue {}: {}",
                issue_id, e
            ))
        })?;

        let transitions: Vec<JiraTransition> = response
            .data
            .as_object()
            .and_then(|obj| obj.get("transitions").cloned())
            .and_then(|t| serde_json::from_value(t).ok())
            .unwrap_or_default();

        // 2. 查找匹配的 transition（忽略大小写）
        let transition = transitions
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(status))
            .ok_or_else(|| {
                JiraError::ApiError(format!(
                    "Status '{}' not found in available transitions for issue {}",
                    status, issue_id
                ))
            })?;

        // 3. 执行状态转换
        let body = TransitionRequest {
            transition: TransitionRef {
                id: transition.id.clone(),
            },
        };
        let body_value = serde_json::to_value(&body).map_err(|e| {
            JiraError::ApiError(format!("Failed to serialize transition request: {}", e))
        })?;
        let path = format!("issue/{}/transitions", issue_id);
        self.jira_client.post(&path, &body_value, None).map_err(|e| {
            JiraError::ApiError(format!(
                "Failed to transition issue {} to status {}: {}",
                issue_id, status, e
            ))
        })?;

        Ok(())
    }

    fn add_comment(&self, issue_id: &str, comment: &str) -> Result<(), JiraError> {
        let body = CommentRequest {
            body: comment.to_string(),
        };
        let body_value = serde_json::to_value(&body).map_err(|e| {
            JiraError::ApiError(format!("Failed to serialize comment request: {}", e))
        })?;
        let path = format!("issue/{}/comment", issue_id);
        self.jira_client.post(&path, &body_value, None).map_err(|e| {
            JiraError::ApiError(format!(
                "Failed to add comment to issue {}: {}",
                issue_id, e
            ))
        })?;

        Ok(())
    }

    fn get_attachments(&self, issue_id: &str) -> Result<Vec<JiraAttachment>, JiraError> {
        let (_, attachments, _) = self.fetch_issue_data(issue_id).map_err(|e| {
            JiraError::ApiError(format!("Failed to get attachments for {}: {}", issue_id, e))
        })?;

        Ok(attachments
            .iter()
            .map(|a| JiraAttachment {
                id: a.filename.clone(), // Jira 附件可能没有单独的 ID，使用 filename 作为标识
                filename: a.filename.clone(),
                size: a.size.unwrap_or(0),
                url: a.content_url.clone(),
            })
            .collect())
    }

    fn fetch_issue_data(
        &self,
        issue_id: &str,
    ) -> Result<(StorageJiraIssue, Vec<StorageJiraAttachment>, Option<String>), JiraError> {
        // 1. 调用 API 获取 issue 信息
        let path = format!("issue/{}?fields=*all&expand=renderedFields", issue_id);
        let response = self
            .jira_client
            .get(&path, None)
            .map_err(|e| JiraError::ApiError(format!("Failed to get issue {}: {}", issue_id, e)))?;

        let issue = response
            .as_model::<StorageJiraIssue>()
            .map_err(|e| JiraError::ApiError(format!("Failed to parse issue data: {}", e)))?;
        let description = issue.fields.description.clone();

        // 2. 从 API 获取附件
        let mut attachments = issue.fields.attachment.clone().unwrap_or_default();

        // 3. 从 description 解析附件链接
        if let Some(ref desc) = description {
            let desc_attachments = self.parse_attachments_from_description(desc);

            // 4. 合并附件（去重）
            for desc_att in desc_attachments {
                if !attachments.iter().any(|a| a.filename == desc_att.filename) {
                    attachments.push(desc_att);
                }
            }
        }

        Ok((issue, attachments, description))
    }
}

impl IssueServiceImpl {
    pub fn new(jira_client: Arc<dyn JiraClient>) -> Self {
        Self { jira_client }
    }

    /// 从描述中解析附件链接
    pub(crate) fn parse_attachments_from_description(
        &self,
        description: &str,
    ) -> Vec<StorageJiraAttachment> {
        let mut attachments = Vec::new();
        let link_pattern = regex::Regex::new(r#"#\s*\[([^|]+)\|([^\]]+)\]"#).unwrap();

        for cap in link_pattern.captures_iter(description) {
            if let (Some(filename_match), Some(url_match)) = (cap.get(1), cap.get(2)) {
                let filename = filename_match.as_str().trim().to_string();
                let url = url_match.as_str().trim().to_string();

                if url.contains("attachments")
                    || filename.ends_with(".txt")
                    || filename.ends_with(".log")
                    || filename.ends_with(".zip")
                {
                    attachments.push(StorageJiraAttachment {
                        filename,
                        content_url: url,
                        mime_type: None,
                        size: None,
                    });
                }
            }
        }

        attachments
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jira::client::types::JiraResponse;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockJiraClient {
        responses: HashMap<String, JiraResponse>,
        last_post: Mutex<Option<(String, serde_json::Value)>>,
    }

    impl MockJiraClient {
        fn new(responses: HashMap<String, JiraResponse>) -> Self {
            Self {
                responses,
                last_post: Mutex::new(None),
            }
        }
    }

    impl JiraClient for MockJiraClient {
        fn get(
            &self,
            path: &str,
            _query: Option<&[(String, String)]>,
        ) -> Result<JiraResponse, JiraError> {
            self.responses
                .get(path)
                .cloned()
                .ok_or_else(|| JiraError::ApiError(format!("missing response for {}", path)))
        }

        fn post(
            &self,
            path: &str,
            body: &serde_json::Value,
            _query: Option<&[(String, String)]>,
        ) -> Result<JiraResponse, JiraError> {
            *self.last_post.lock().unwrap() = Some((path.to_string(), body.clone()));
            Ok(JiraResponse::new(json!({})))
        }
    }

    fn build_issue_response() -> JiraResponse {
        JiraResponse::new(json!({
            "key": "PROJ-1",
            "id": "10000",
            "self": "https://jira.example.com/rest/api/2/issue/10000",
            "fields": {
                "summary": "Summary",
                "description": "See # [log.txt|https://jira.example.com/attachments/log.txt]",
                "status": {
                    "id": "1",
                    "name": "Open",
                    "self": "https://jira.example.com/status/1"
                },
                "attachment": [{
                    "filename": "data.zip",
                    "content": "https://jira.example.com/attachments/data.zip",
                    "mimeType": "application/zip",
                    "size": 10
                }],
                "comment": {
                    "comments": [{
                        "id": "1",
                        "body": "hello",
                        "created": "now",
                        "author": {
                            "accountId": "123",
                            "displayName": "User"
                        }
                    }]
                },
                "labels": ["bug"],
                "components": [{ "id": "1", "name": "core" }]
            }
        }))
    }

    #[test]
    fn test_get_issue_info_maps_fields() {
        let mut responses = HashMap::new();
        responses.insert(
            "issue/PROJ-1?fields=*all&expand=renderedFields".to_string(),
            build_issue_response(),
        );
        let client = Arc::new(MockJiraClient::new(responses));
        let service = IssueServiceImpl::new(client);

        let issue = service.get_issue_info("PROJ-1").unwrap();
        assert_eq!(issue.key, "PROJ-1");
        assert_eq!(issue.status, "Open");
        assert_eq!(issue.attachments.len(), 2);
    }

    #[test]
    fn test_update_issue_status_posts_transition() {
        let transitions = JiraResponse::new(json!({
            "transitions": [
                { "id": "31", "name": "Done" }
            ]
        }));
        let mut responses = HashMap::new();
        responses.insert("issue/PROJ-1/transitions".to_string(), transitions);
        let client = Arc::new(MockJiraClient::new(responses));
        let service = IssueServiceImpl::new(client.clone());

        service.update_issue_status("PROJ-1", "Done").unwrap();

        let (path, body) = client.last_post.lock().unwrap().clone().unwrap();
        assert_eq!(path, "issue/PROJ-1/transitions");
        assert_eq!(body, json!({ "transition": { "id": "31" } }));
    }

    #[test]
    fn test_parse_attachments_from_description() {
        let client = Arc::new(MockJiraClient::new(HashMap::new()));
        let service = IssueServiceImpl::new(client);
        let description = "See # [log.txt|https://jira.example.com/attachments/log.txt]";
        let attachments = service.parse_attachments_from_description(description);
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].filename, "log.txt");
    }
}
