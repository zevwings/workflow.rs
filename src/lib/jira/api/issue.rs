//! Jira Issue/Ticket REST API
//!
//! 本模块提供了所有 Issue/Ticket 相关的 REST API 方法。

use color_eyre::{
    eyre::{ContextCompat, WrapErr},
    Result,
};
use reqwest::blocking::multipart;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

use super::helpers::{build_jira_url, jira_auth_config};
use crate::base::http::{HttpClient, MultipartRequestConfig, RequestConfig};
use crate::base::util::FileReader;
use crate::jira::types::{
    JiraAttachment, JiraChangelog, JiraChangelogHistory, JiraChangelogItem, JiraIssue,
    JiraTransition,
};

/// 状态转换请求体
///
/// 用于更新 issue 状态的请求体结构。
#[derive(Serialize)]
struct TransitionRequest {
    transition: TransitionRef,
}

/// 状态转换引用
///
/// 包含要执行的状态转换的 ID。
#[derive(Serialize)]
struct TransitionRef {
    id: String,
}

/// 分配请求体
///
/// 用于分配 issue 给用户的请求体结构。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AssigneeRequest {
    account_id: String,
}

/// 评论请求体
///
/// 用于添加评论到 issue 的请求体结构。
#[derive(Serialize)]
struct CommentRequest {
    body: String,
}

pub struct JiraIssueApi;

impl JiraIssueApi {
    /// 获取 issue 信息
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    ///
    /// # 返回
    ///
    /// 返回 `JiraIssue` 结构体，包含 ticket 的所有信息。
    pub fn get_issue(ticket: &str) -> Result<JiraIssue> {
        let url = build_jira_url(&format!(
            "issue/{}?fields=*all&expand=renderedFields",
            ticket
        ))?;
        let client = HttpClient::global()?;
        let auth = jira_auth_config()?;
        let config = RequestConfig::<Value, Value>::new().auth(auth);
        let response = client.get(&url, config)?;
        response
            .ensure_success()?
            .as_json()
            .wrap_err(format!("Failed to get issue: {}", ticket))
    }

    /// 获取 issue 的附件列表
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    ///
    /// # 返回
    ///
    /// 返回附件列表，如果没有附件则返回空列表。
    pub fn get_issue_attachments(ticket: &str) -> Result<Vec<JiraAttachment>> {
        let issue = Self::get_issue(ticket)?;
        Ok(issue.fields.attachment.unwrap_or_default())
    }

    /// 获取 issue 的可用 transitions
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    ///
    /// # 返回
    ///
    /// 返回可用的 transitions 列表，每个 transition 包含 ID 和名称。
    pub fn get_issue_transitions(ticket: &str) -> Result<Vec<JiraTransition>> {
        let url = build_jira_url(&format!("issue/{}/transitions", ticket))?;
        let client = HttpClient::global()?;
        let auth = jira_auth_config()?;
        let config = RequestConfig::<Value, Value>::new().auth(auth);
        let response = client.get(&url, config)?;
        let data: Value = response
            .ensure_success()?
            .as_json()
            .wrap_err(format!("Failed to get transitions for ticket: {}", ticket))?;

        let transitions = data
            .get("transitions")
            .and_then(|t| t.as_array())
            .wrap_err("Invalid transitions JSON structure")?;

        let result: Vec<JiraTransition> = transitions
            .iter()
            .filter_map(|t| {
                let id = t.get("id")?.as_str()?.to_string();
                let name = t.get("name")?.as_str()?.to_string();
                Some(JiraTransition { id, name })
            })
            .collect();

        Ok(result)
    }

    /// 更新 issue 状态
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    /// * `transition_id` - Transition ID
    ///
    /// # 返回
    ///
    /// 成功时返回 `Ok(())`。
    pub fn transition_issue(ticket: &str, transition_id: &str) -> Result<()> {
        let url = build_jira_url(&format!("issue/{}/transitions", ticket))?;
        let client = HttpClient::global()?;
        let auth = jira_auth_config()?;

        let body = TransitionRequest {
            transition: TransitionRef {
                id: transition_id.to_string(),
            },
        };

        let config = RequestConfig::<TransitionRequest, Value>::new().body(&body).auth(auth);
        let response = client.post(&url, config)?;
        response.ensure_success().wrap_err(format!(
            "Failed to transition issue {} to transition {}",
            ticket, transition_id
        ))?;
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
    pub fn assign_issue(ticket: &str, account_id: &str) -> Result<()> {
        let url = build_jira_url(&format!("issue/{}/assignee", ticket))?;
        let client = HttpClient::global()?;
        let auth = jira_auth_config()?;

        let body = AssigneeRequest {
            account_id: account_id.to_string(),
        };

        let config = RequestConfig::<AssigneeRequest, Value>::new().body(&body).auth(auth);
        let response = client.put(&url, config)?;
        response.ensure_success().wrap_err(format!(
            "Failed to assign issue {} to {}",
            ticket, account_id
        ))?;
        Ok(())
    }

    /// 添加评论到 issue
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    /// * `comment` - 评论内容
    ///
    /// # 返回
    ///
    /// 成功时返回 `Ok(())`。
    pub fn add_issue_comment(ticket: &str, comment: &str) -> Result<()> {
        let url = build_jira_url(&format!("issue/{}/comment", ticket))?;
        let client = HttpClient::global()?;
        let auth = jira_auth_config()?;

        let body = CommentRequest {
            body: comment.to_string(),
        };

        let config = RequestConfig::<CommentRequest, Value>::new().body(&body).auth(auth);
        let response = client.post(&url, config)?;
        response
            .ensure_success()
            .wrap_err(format!("Failed to add comment to issue {}", ticket))?;
        Ok(())
    }

    /// 上传附件到 issue
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    /// * `file_path` - 要上传的文件路径
    ///
    /// # 返回
    ///
    /// 成功时返回上传的附件信息列表（JIRA API 返回的是数组）。
    ///
    /// # 错误
    ///
    /// 如果文件不存在、无法读取或上传失败，返回相应的错误信息。
    pub fn upload_attachment(ticket: &str, file_path: &str) -> Result<Vec<JiraAttachment>> {
        let url = build_jira_url(&format!("issue/{}/attachments", ticket))?;
        let auth = jira_auth_config()?;

        // 读取文件
        let path = Path::new(file_path);
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .wrap_err_with(|| format!("Invalid file name: {}", file_path))?;

        let file_data = FileReader::new(path).bytes()?;

        // 创建 multipart form
        // 根据文件扩展名确定 MIME 类型
        let mime_type = Self::guess_mime_type(path);
        let part = multipart::Part::bytes(file_data)
            .file_name(file_name.to_string())
            .mime_str(&mime_type)
            .wrap_err_with(|| format!("Failed to create multipart part for: {}", file_path))?;

        let form = multipart::Form::new().part("file", part);

        // 设置自定义 headers（Jira 需要 X-Atlassian-Token）
        let mut headers = HeaderMap::new();
        headers.insert("X-Atlassian-Token", HeaderValue::from_static("no-check"));

        // 使用 HttpClient 发送 multipart 请求
        let client = HttpClient::global()?;
        let config = MultipartRequestConfig::<Value>::new()
            .multipart(form)
            .auth(auth.clone())
            .headers(headers);

        let response = client
            .post_multipart(&url, config)
            .wrap_err_with(|| format!("Failed to upload attachment to issue {}", ticket))?;

        // 检查响应状态并解析
        let response = response.ensure_success_with(|r| {
            color_eyre::eyre::eyre!(
                "Failed to upload attachment: HTTP {} - {}",
                r.status,
                r.extract_error_message()
            )
        })?;

        let attachments: Vec<JiraAttachment> =
            response.as_json().wrap_err_with(|| "Failed to parse attachment response")?;

        Ok(attachments)
    }

    /// 根据文件路径猜测 MIME 类型
    ///
    /// # 参数
    ///
    /// * `path` - 文件路径
    ///
    /// # 返回
    ///
    /// 返回 MIME 类型字符串，如果无法确定则返回 "application/octet-stream"
    fn guess_mime_type(path: &Path) -> String {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "pdf" => "application/pdf",
            "txt" => "text/plain",
            "json" => "application/json",
            "xml" => "application/xml",
            "zip" => "application/zip",
            "py" => "text/x-python",
            "rs" => "text/x-rust",
            "js" => "text/javascript",
            "html" => "text/html",
            "css" => "text/css",
            "md" => "text/markdown",
            _ => "application/octet-stream",
        }
        .to_string()
    }

    /// 获取 issue 的变更历史（changelog）
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    ///
    /// # 返回
    ///
    /// 返回 `JiraChangelog` 结构体，包含所有变更记录。
    ///
    /// # 错误
    ///
    /// 如果 ticket 不存在或无法访问，返回错误。
    pub fn get_issue_changelog(ticket: &str) -> Result<JiraChangelog> {
        // 先验证 ticket 是否存在
        let _issue = Self::get_issue(ticket)
            .wrap_err_with(|| format!("Ticket {} does not exist or is not accessible", ticket))?;

        // 使用专门的 changelog 端点
        // API v2: /rest/api/2/issue/{issueIdOrKey}/changelog
        let url = build_jira_url(&format!("issue/{}/changelog", ticket))?;
        let client = HttpClient::global()?;
        let auth = jira_auth_config()?;
        let config = RequestConfig::<Value, Value>::new().auth(auth);
        let response = client.get(&url, config)?;
        let data: Value = response
            .ensure_success()
            .wrap_err_with(|| format!("Failed to get changelog for ticket: {}. The ticket may not exist or you may not have permission to view it.", ticket))?
            .as_json()
            .wrap_err(format!("Failed to parse changelog response for ticket: {}", ticket))?;

        // 提取 changelog 数据
        // API v2 返回格式: { "id": "...", "histories": [...] }
        let histories = data
            .get("values")
            .or_else(|| data.get("histories"))
            .and_then(|h| h.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|h| {
                let id = h.get("id")?.as_str()?.to_string();
                let created = h.get("created")?.as_str()?.to_string();
                let author = h.get("author").and_then(|a| serde_json::from_value(a.clone()).ok());
                let items: Vec<JiraChangelogItem> = h
                    .get("items")
                    .and_then(|i| i.as_array())
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|item| serde_json::from_value(item.clone()).ok())
                    .collect();

                Some(JiraChangelogHistory {
                    id,
                    created,
                    author,
                    items,
                })
            })
            .collect();

        Ok(JiraChangelog {
            id: ticket.to_string(),
            histories,
        })
    }
}
