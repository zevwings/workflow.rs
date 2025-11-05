use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::commands::Jira;
use crate::settings::Settings;

/// Jira REST API 模块
pub struct JiraApi;

impl JiraApi {
    /// 获取项目状态列表（通过 REST API）
    pub fn get_project_statuses(project: &str) -> Result<Vec<String>> {
        let email = Jira::get_current_user()?;
        let settings = Settings::load();
        let api_token = &settings.jira_api_token;
        let service_address = &settings.jira_service_address;

        let url = format!(
            "{}/rest/api/2/project/{}/statuses",
            service_address, project
        );

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&url)
            .basic_auth(&email, Some(&api_token))
            .send()
            .context("Failed to fetch project statuses")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch project statuses: {}", response.status());
        }

        let json: serde_json::Value = response
            .json()
            .context("Failed to parse project statuses JSON")?;

        // 解析状态列表（取第一个版本的 statuses）
        let statuses = json
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
        let email = Jira::get_current_user()?;
        let settings = Settings::load();
        let api_token = &settings.jira_api_token;
        let service_address = &settings.jira_service_address;

        let url = format!("{}/rest/api/2/issue/{}", service_address, ticket);

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&url)
            .basic_auth(&email, Some(&api_token))
            .send()
            .context("Failed to fetch Jira ticket")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch Jira ticket: {}", response.status());
        }

        let json: serde_json::Value = response
            .json()
            .context("Failed to parse Jira ticket JSON")?;

        let summary = json
            .get("fields")
            .and_then(|f| f.get("summary"))
            .and_then(|s| s.as_str())
            .context("Failed to extract summary from Jira ticket")?;

        Ok(summary.to_string())
    }

    /// 下载附件（用于日志下载功能）
    pub fn get_attachments(ticket: &str) -> Result<Vec<JiraAttachment>> {
        let email = Jira::get_current_user()?;
        let settings = Settings::load();
        let api_token = &settings.jira_api_token;
        let service_address = &settings.jira_service_address;

        let url = format!("{}/rest/api/2/issue/{}", service_address, ticket);

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&url)
            .basic_auth(&email, Some(&api_token))
            .send()
            .context("Failed to fetch ticket info")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch ticket info: {}", response.status());
        }

        let json: serde_json::Value = response.json().context("Failed to parse ticket JSON")?;

        // 解析附件列表
        let attachments = json
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
}

/// Jira 附件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraAttachment {
    pub filename: String,
    pub content_url: String,
    pub mime_type: Option<String>,
    pub size: Option<u64>,
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
