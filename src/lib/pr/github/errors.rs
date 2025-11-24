use crate::base::http::HttpResponse;
use anyhow::Error;
use serde::Deserialize;
use serde_json::Value;

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

/// 格式化 GitHub 错误信息
///
/// 将 GitHub API 错误响应格式化为用户友好的错误消息
pub fn format_error(error: &GitHubErrorResponse, response: &HttpResponse) -> Error {
    let mut msg = format!(
        "GitHub API error: {} (Status: {})",
        error.message, response.status
    );

    if let Some(errors) = &error.errors {
        for err in errors {
            if let (Some(resource), Some(field), Some(code)) =
                (&err.resource, &err.field, &err.code)
            {
                msg.push_str(&format!(
                    "\n  - {}: {} field is invalid ({})",
                    resource, field, code
                ));
            }
        }
    }

    // 尝试添加完整的错误响应 JSON 以便调试
    if let Ok(data) = response.as_json::<Value>() {
        if let Ok(json_str) = serde_json::to_string_pretty(&data) {
            msg.push_str(&format!("\n\nFull error response:\n{}", json_str));
        }
    }

    anyhow::anyhow!(msg)
}

/// 处理 GitHub API 错误
///
/// 尝试解析 GitHub 错误格式，如果无法解析则返回通用错误信息
pub fn handle_github_error(response: &HttpResponse) -> Error {
    // 尝试解析 JSON 错误
    if let Ok(data) = response.as_json::<Value>() {
        // 尝试解析为 GitHub 错误格式
        if let Ok(error) = serde_json::from_value::<GitHubErrorResponse>(data.clone()) {
            return format_error(&error, response);
        }

        // 如果无法解析为 GitHub 格式，返回 JSON 字符串
        if let Ok(json_str) = serde_json::to_string_pretty(&data) {
            return anyhow::anyhow!(
                "GitHub API request failed: {} - {}\n\nResponse:\n{}",
                response.status,
                response.status_text,
                json_str
            );
        }
    }

    // 回退到简单错误
    anyhow::anyhow!(
        "GitHub API request failed: {} - {}",
        response.status,
        response.status_text
    )
}
