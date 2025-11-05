use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::http::{Authorization, HttpClient};
use crate::settings::Settings;

/// Jira REST API 客户端
pub struct JiraClient;

impl JiraClient {
    /// 获取认证信息（email 和 api_token）
    fn get_auth() -> Result<(String, String)> {
        let settings = Settings::load();
        let email = settings.email.clone();
        let api_token = settings.jira_api_token.clone();
        Ok((email, api_token))
    }

    /// 获取当前 Jira 用户邮箱
    pub fn get_current_user() -> Result<String> {
        let (email, api_token) = Self::get_auth()?;
        let settings = Settings::load();
        let service_address = &settings.jira_service_address;

        let url = format!("{}/rest/api/2/myself", service_address);

        let client = HttpClient::new()?;
        let auth = Authorization::new(&email, &api_token);
        let response = client
            .get::<serde_json::Value>(&url, Some(&auth), None)
            .context("Failed to get current Jira user")?;

        if !response.is_success() {
            anyhow::bail!("Failed to get current Jira user: {}", response.status);
        }

        let email_address = response
            .data
            .get("emailAddress")
            .and_then(|e| e.as_str())
            .context("Failed to extract emailAddress from Jira user")?;

        Ok(email_address.to_string())
    }

    /// 获取 ticket 信息
    pub fn get_ticket_info(ticket: &str) -> Result<serde_json::Value> {
        let (email, api_token) = Self::get_auth()?;
        let settings = Settings::load();
        let service_address = &settings.jira_service_address;

        let url = format!("{}/rest/api/2/issue/{}", service_address, ticket);

        let client = HttpClient::new()?;
        let auth = Authorization::new(&email, &api_token);
        let response = client
            .get::<serde_json::Value>(&url, Some(&auth), None)
            .context(format!("Failed to get ticket info: {}", ticket))?;

        if !response.is_success() {
            anyhow::bail!("Failed to get ticket info: {}", response.status);
        }

        Ok(response.data)
    }

    /// 获取项目状态列表（通过 REST API）
    pub fn get_project_statuses(project: &str) -> Result<Vec<String>> {
        let (email, api_token) = Self::get_auth()?;
        let settings = Settings::load();
        let service_address = &settings.jira_service_address;

        let url = format!(
            "{}/rest/api/2/project/{}/statuses",
            service_address, project
        );

        let client = HttpClient::new()?;
        let auth = Authorization::new(&email, &api_token);
        let response = client
            .get::<serde_json::Value>(&url, Some(&auth), None)
            .context("Failed to fetch project statuses")?;

        if !response.is_success() {
            anyhow::bail!("Failed to fetch project statuses: {}", response.status);
        }

        // 解析状态列表（取第一个版本的 statuses）
        let statuses = response
            .data
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|version| version.get("statuses"))
            .and_then(|s| s.as_array())
            .context("Invalid statuses JSON structure")?;

        let status_names: Vec<String> = statuses
            .iter()
            .filter_map(|s| s.get("name"))
            .filter_map(|n| n.as_str())
            .map(|s| s.to_string())
            .collect();

        Ok(status_names)
    }

    /// 从 Jira REST API 获取 issue summary
    pub fn get_summary(ticket: &str) -> Result<String> {
        let (email, api_token) = Self::get_auth()?;
        let settings = Settings::load();
        let service_address = &settings.jira_service_address;

        let url = format!("{}/rest/api/2/issue/{}", service_address, ticket);

        let client = HttpClient::new()?;
        let auth = Authorization::new(&email, &api_token);
        let response = client
            .get::<serde_json::Value>(&url, Some(&auth), None)
            .context("Failed to fetch Jira ticket")?;

        if !response.is_success() {
            anyhow::bail!("Failed to fetch Jira ticket: {}", response.status);
        }

        let summary = response
            .data
            .get("fields")
            .and_then(|f| f.get("summary"))
            .and_then(|s| s.as_str())
            .context("Failed to extract summary from Jira ticket")?;

        Ok(summary.to_string())
    }

    /// 下载附件（用于日志下载功能）
    pub fn get_attachments(ticket: &str) -> Result<Vec<JiraAttachment>> {
        let (email, api_token) = Self::get_auth()?;
        let settings = Settings::load();
        let service_address = &settings.jira_service_address;

        let url = format!("{}/rest/api/2/issue/{}", service_address, ticket);

        let client = HttpClient::new()?;
        let auth = Authorization::new(&email, &api_token);
        let response = client
            .get::<serde_json::Value>(&url, Some(&auth), None)
            .context("Failed to fetch ticket info")?;

        if !response.is_success() {
            anyhow::bail!("Failed to fetch ticket info: {}", response.status);
        }

        // 解析附件列表
        let attachments = response
            .data
            .get("fields")
            .and_then(|f| f.get("attachment"))
            .and_then(|a| a.as_array())
            .context("No attachments found or invalid JSON structure")?;

        let result: Vec<JiraAttachment> = attachments
            .iter()
            .filter_map(|a| {
                let filename = a.get("filename")?.as_str()?.to_string();
                let content_url = a.get("content")?.as_str()?.to_string();
                let mime_type = a
                    .get("mimeType")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string());
                let size = a.get("size").and_then(|s| s.as_u64());

                Some(JiraAttachment {
                    filename,
                    content_url,
                    mime_type,
                    size,
                })
            })
            .collect();

        Ok(result)
    }

    /// 获取 issue 的可用 transitions（用于状态转换）
    fn get_transitions(ticket: &str) -> Result<Vec<Transition>> {
        let (email, api_token) = Self::get_auth()?;
        let settings = Settings::load();
        let service_address = &settings.jira_service_address;

        let url = format!(
            "{}/rest/api/2/issue/{}/transitions",
            service_address, ticket
        );

        let client = HttpClient::new()?;
        let auth = Authorization::new(&email, &api_token);
        let response = client
            .get::<serde_json::Value>(&url, Some(&auth), None)
            .context(format!("Failed to get transitions for ticket: {}", ticket))?;

        if !response.is_success() {
            anyhow::bail!("Failed to get transitions: {}", response.status);
        }

        let transitions = response
            .data
            .get("transitions")
            .and_then(|t| t.as_array())
            .context("Invalid transitions JSON structure")?;

        let result: Vec<Transition> = transitions
            .iter()
            .filter_map(|t| {
                let id = t.get("id")?.as_str()?.to_string();
                let name = t.get("name")?.as_str()?.to_string();
                Some(Transition { id, name })
            })
            .collect();

        Ok(result)
    }

    /// 更新 ticket 状态
    pub fn move_ticket(ticket: &str, status: &str) -> Result<()> {
        let (email, api_token) = Self::get_auth()?;
        let settings = Settings::load();
        let service_address = &settings.jira_service_address;

        // 先获取可用的 transitions
        let transitions = Self::get_transitions(ticket)?;

        // 查找匹配的 transition
        let transition = transitions
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(status))
            .context(format!(
                "Status '{}' not found in available transitions for ticket {}",
                status, ticket
            ))?;

        let url = format!(
            "{}/rest/api/2/issue/{}/transitions",
            service_address, ticket
        );

        #[derive(Serialize)]
        struct TransitionRequest {
            transition: TransitionRef,
        }

        #[derive(Serialize)]
        struct TransitionRef {
            id: String,
        }

        let body = TransitionRequest {
            transition: TransitionRef {
                id: transition.id.clone(),
            },
        };

        let client = HttpClient::new()?;
        let auth = Authorization::new(&email, &api_token);
        let response = client
            .post::<serde_json::Value, _>(&url, &body, Some(&auth), None)
            .context(format!(
                "Failed to move ticket {} to status {}",
                ticket, status
            ))?;

        if !response.is_success() {
            anyhow::bail!(
                "Failed to move ticket {} to status {}: {}",
                ticket,
                status,
                response.status
            );
        }

        Ok(())
    }

    /// 分配 ticket 给用户
    pub fn assign_ticket(ticket: &str, assignee: Option<&str>) -> Result<()> {
        let (email, api_token) = Self::get_auth()?;
        let settings = Settings::load();
        let service_address = &settings.jira_service_address;

        let assignee_email = match assignee {
            Some(user) => user.to_string(),
            None => {
                // 如果没有指定，分配给当前用户
                Self::get_current_user()?
            }
        };

        let url = format!("{}/rest/api/2/issue/{}/assignee", service_address, ticket);

        #[derive(Serialize)]
        struct AssigneeRequest {
            name: String,
        }

        let body = AssigneeRequest {
            name: assignee_email.clone(),
        };

        let client = HttpClient::new()?;
        let auth = Authorization::new(&email, &api_token);
        let response = client
            .put::<serde_json::Value, _>(&url, &body, Some(&auth), None)
            .context(format!(
                "Failed to assign ticket {} to {}",
                ticket, assignee_email
            ))?;

        if !response.is_success() {
            anyhow::bail!(
                "Failed to assign ticket {} to {}: {}",
                ticket,
                assignee_email,
                response.status
            );
        }

        Ok(())
    }

    /// 添加评论到 ticket
    pub fn add_comment(ticket: &str, comment: &str) -> Result<()> {
        let (email, api_token) = Self::get_auth()?;
        let settings = Settings::load();
        let service_address = &settings.jira_service_address;

        let url = format!("{}/rest/api/2/issue/{}/comment", service_address, ticket);

        #[derive(Serialize)]
        struct CommentRequest {
            body: String,
        }

        let body = CommentRequest {
            body: comment.to_string(),
        };

        let client = HttpClient::new()?;
        let auth = Authorization::new(&email, &api_token);
        let response = client
            .post::<serde_json::Value, _>(&url, &body, Some(&auth), None)
            .context(format!("Failed to add comment to ticket {}", ticket))?;

        if !response.is_success() {
            anyhow::bail!(
                "Failed to add comment to ticket {}: {}",
                ticket,
                response.status
            );
        }

        Ok(())
    }
}

/// Jira 附件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraAttachment {
    pub filename: String,
    pub content_url: String,
    pub mime_type: Option<String>,
    pub size: Option<u64>,
}

/// Transition 信息
#[derive(Debug, Clone)]
struct Transition {
    id: String,
    name: String,
}

/// 从 PR 标题提取 Jira ticket ID
///
/// # 示例
/// ```
/// assert_eq!(extract_jira_ticket_id("PROJ-123: Fix bug"), Some("PROJ-123"));
/// assert_eq!(extract_jira_ticket_id("Fix bug"), None);
/// ```
pub fn extract_jira_ticket_id(pr_title: &str) -> Option<String> {
    use regex::Regex;
    // 匹配格式: PROJ-123 或 PROJ-123:
    let re = Regex::new(r"^([A-Z]+-\d+)").ok()?;
    re.captures(pr_title)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_jira_ticket_id() {
        assert_eq!(
            extract_jira_ticket_id("PROJ-123: Fix bug"),
            Some("PROJ-123".to_string())
        );
        assert_eq!(
            extract_jira_ticket_id("ABC-456 Description"),
            Some("ABC-456".to_string())
        );
        assert_eq!(extract_jira_ticket_id("Fix bug"), None);
    }
}

