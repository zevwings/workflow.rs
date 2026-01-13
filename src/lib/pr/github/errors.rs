use crate::base::http::HttpResponse;
use color_eyre::eyre::Report;
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

/// GitHub 错误响应结构
#[derive(Debug, Deserialize)]
pub struct GitHubErrorResponse {
    pub message: String,
    pub errors: Option<Vec<GitHubError>>,
}

/// GitHub 错误详情
#[derive(Debug, Deserialize)]
pub struct GitHubError {
    pub resource: Option<String>,
    pub field: Option<String>,
    pub code: Option<String>,
}

/// GitHub API 错误类型
#[derive(Debug, Error)]
pub enum GitHubApiError {
    /// GitHub API 错误（带详细错误信息）
    #[error("GitHub API error: {message} (Status: {status}){details}")]
    ApiError {
        message: String,
        status: u16,
        details: String,
    },
    /// GitHub API 请求失败（无法解析为 GitHub 错误格式）
    #[error("GitHub API request failed: {status} - {status_text}\n\nResponse:\n{response}")]
    RequestFailed {
        status: u16,
        status_text: String,
        response: String,
    },
    /// GitHub API 请求失败（简单错误）
    #[error("GitHub API request failed: {status} - {status_text}")]
    SimpleRequestFailed { status: u16, status_text: String },
}

/// 格式化 GitHub 错误信息
///
/// 将 GitHub API 错误响应格式化为用户友好的错误消息
pub fn format_error(error: &GitHubErrorResponse, response: &HttpResponse) -> GitHubApiError {
    let mut details = String::new();

    if let Some(errors) = &error.errors {
        for err in errors {
            if let (Some(resource), Some(field), Some(code)) =
                (&err.resource, &err.field, &err.code)
            {
                details.push_str(&format!(
                    "\n  - {}: {} field is invalid ({})",
                    resource, field, code
                ));
            }
        }
    }

    // 尝试添加完整的错误响应 JSON 以便调试
    if let Ok(data) = response.as_json::<Value>() {
        if let Ok(json_str) = serde_json::to_string_pretty(&data) {
            details.push_str(&format!("\n\nFull error response:\n{}", json_str));
        }
    } else {
        // 如果无法解析为 JSON，添加提取的错误消息
        let error_msg = response.extract_error_message();
        if !error_msg.is_empty() {
            details.push_str(&format!("\n\nError details:\n{}", error_msg));
        }
    }

    GitHubApiError::ApiError {
        message: error.message.clone(),
        status: response.status,
        details,
    }
}

/// 处理 GitHub API 错误
///
/// 尝试解析 GitHub 错误格式，如果无法解析则返回通用错误信息
///
/// 返回 `Report` 以保持与现有调用方的兼容性（`ensure_success_with` 需要 `Into<Report>`）
pub fn handle_github_error(response: &HttpResponse) -> Report {
    // 尝试解析 JSON 错误
    if let Ok(data) = response.as_json::<Value>() {
        // 尝试解析为 GitHub 错误格式
        if let Ok(error) = serde_json::from_value::<GitHubErrorResponse>(data.clone()) {
            return format_error(&error, response).into();
        }

        // 如果无法解析为 GitHub 格式，返回 JSON 字符串
        if let Ok(json_str) = serde_json::to_string_pretty(&data) {
            return GitHubApiError::RequestFailed {
                status: response.status,
                status_text: response.status_text.clone(),
                response: json_str,
            }
            .into();
        }
    }

    // 回退到简单错误
    GitHubApiError::SimpleRequestFailed {
        status: response.status,
        status_text: response.status_text.clone(),
    }
    .into()
}
