//! Issue 数据获取服务
//!
//! 提供统一的 Issue 数据获取和操作功能，包括：
//! - 从 API 获取 Issue 数据
//! - 解析附件（从 API 和描述中）
//! - 合并和去重附件
//! - Domain 类型转换
//! - Issue 操作（状态更新、添加评论）

use std::sync::Arc;

use domain::{JiraAttachment, JiraError, JiraIssue, JiraTransition};
use regex::Regex;
use serde::Serialize;

use crate::jira::client::{core::JiraClient, types::JiraResponseSerializable};

// 文件私有请求类型

#[derive(Serialize)]
struct TransitionRequest {
    transition: TransitionRef,
}

#[derive(Serialize)]
struct TransitionRef {
    id: String,
}

/// 分配请求体
///
/// 用于分配 issue 给用户的请求体结构。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssigneeRequest {
    pub account_id: String,
}

#[derive(Serialize)]
struct CommentRequest {
    body: String,
}

pub trait IssueService: Send + Sync {
    fn get_issue_info(&self, issue_id: &str) -> Result<JiraIssue, JiraError>;
    fn update_issue_status(&self, issue_id: &str, status: &str) -> Result<(), JiraError>;
    fn assign_issue(&self, ticket: &str, account_id: &str) -> Result<(), JiraError>;
    fn add_comment(&self, issue_id: &str, comment: &str) -> Result<(), JiraError>;
    fn get_attachments(&self, issue_id: &str) -> Result<Vec<JiraAttachment>, JiraError>;
    fn fetch_issue_data(
        &self,
        issue_id: &str,
    ) -> Result<(JiraIssue, Vec<JiraAttachment>, Option<String>), JiraError>;
}

/// Issue 数据获取服务
///
/// 提供统一的 Issue 数据获取功能，避免重复的 API 调用。
pub struct IssueServiceImpl {
    jira_client: Arc<dyn JiraClient>,
}

impl IssueService for IssueServiceImpl {
    fn get_issue_info(&self, issue_id: &str) -> Result<JiraIssue, JiraError> {
        let (issue, _, _) = self
            .fetch_issue_data(issue_id)
            .map_err(|e| JiraError::ApiError(format!("Failed to get issue {}: {}", issue_id, e)))?;

        // DTO → Domain 映射
        Ok(issue)
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

    /// 分配 issue 给用户
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    /// * `account_id` - 被分配用户的 account_id
    ///
    /// # 返回
    ///
    /// 成功时返回 `Ok(())`。
    fn assign_issue(&self, ticket: &str, account_id: &str) -> Result<(), JiraError> {
        let path = format!("issue/{}/assignee", ticket);
        let body = AssigneeRequest {
            account_id: account_id.to_string(),
        };
        let body_value = serde_json::to_value(&body).map_err(|e| {
            JiraError::ApiError(format!("Failed to serialize assignee request: {}", e))
        })?;
        self.jira_client.put(&path, &body_value, None).map_err(|e| {
            JiraError::ApiError(format!("Failed to assign issue {}: {}", ticket, e))
        })?;
        Ok(())
    }

    /// 添加评论
    ///
    /// # 参数
    ///
    /// * `issue_id` - Jira ticket ID，格式如 `PROJ-123`
    /// * `comment` - 评论内容
    ///
    /// # 返回
    ///
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

        Ok(attachments)
    }

    fn fetch_issue_data(
        &self,
        issue_id: &str,
    ) -> Result<(JiraIssue, Vec<JiraAttachment>, Option<String>), JiraError> {
        // 1. 调用 API 获取 issue 信息
        let path = format!("issue/{}?fields=*all&expand=renderedFields", issue_id);
        let response = self
            .jira_client
            .get(&path, None)
            .map_err(|e| JiraError::ApiError(format!("Failed to get issue {}: {}", issue_id, e)))?;

        let issue = response
            .as_model::<JiraIssue>()
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
    ) -> Vec<JiraAttachment> {
        let mut attachments = Vec::new();
        let link_pattern = Regex::new(r#"#\s*\[([^|]+)\|([^\]]+)\]"#).unwrap();

        for cap in link_pattern.captures_iter(description) {
            if let (Some(filename_match), Some(url_match)) = (cap.get(1), cap.get(2)) {
                let filename = filename_match.as_str().trim().to_string();
                let url = url_match.as_str().trim().to_string();

                if url.contains("attachments")
                    || filename.ends_with(".txt")
                    || filename.ends_with(".log")
                    || filename.ends_with(".zip")
                {
                    attachments.push(JiraAttachment {
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
