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
