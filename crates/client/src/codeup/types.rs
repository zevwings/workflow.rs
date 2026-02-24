//! Codeup API 响应和错误处理

use crate::{CodeupClientError, HttpResponse};
use serde::{Deserialize, Serialize};

/// HTTP 响应格式
#[derive(Debug)]
pub struct CodeupResponse {
    response: HttpResponse,
}

impl CodeupResponse {
    pub fn new(response: HttpResponse) -> Self {
        Self { response }
    }

    /// 解析为 JSON
    pub fn json<T>(&self) -> Result<T, crate::HttpError>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.response.json()
    }

    /// 解析为文本
    pub fn text(&self) -> Result<&str, CodeupClientError> {
        self.response.text().map_err(|e| CodeupClientError::ApiError(e.to_string()))
    }

    pub fn status_code(&self) -> u16 {
        self.response.status
    }
}

/// Codeup 错误响应结构
#[derive(Debug, Deserialize)]
pub struct CodeupErrorResponse {
    pub message: String,
    pub error: Option<String>,
}

/// Codeup PR 创建请求
#[derive(Debug, Serialize)]
pub struct CreateCodeupPullRequestRequest {
    pub title: String,
    pub description: String,
    pub source_branch: String,
    pub target_branch: String,
}

/// Codeup PR 创建响应
#[derive(Debug, Deserialize)]
pub struct CreateCodeupPullRequestResponse {
    pub id: i64,
    pub iid: i64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub source_branch: String,
    pub target_branch: String,
    pub web_url: String,
}

/// Codeup PR 列表响应
#[derive(Debug, Deserialize)]
pub struct CodeupPullRequestListResponse {
    pub id: i64,
    pub iid: i64,
    pub title: String,
    pub state: String,
    pub source_branch: String,
    pub target_branch: String,
    pub web_url: String,
}
