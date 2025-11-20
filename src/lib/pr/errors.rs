use crate::base::http::HttpResponse;
use crate::pr::codeup::errors::CodeupErrorResponse;
use crate::pr::github::errors::{format_error as format_github_error, GitHubErrorResponse};
use anyhow::Error;
use serde_json::Value;

/// 统一的 API 错误处理
///
/// 尝试解析不同平台的错误格式，提供详细的错误信息
pub fn handle_api_error(response: &HttpResponse) -> Error {
    // 尝试解析 JSON 错误
    if let Ok(data) = response.as_json::<Value>() {
        // 尝试解析为 GitHub 错误格式
        if let Ok(error) = serde_json::from_value::<GitHubErrorResponse>(data.clone()) {
            return format_github_error(&error, response);
        }
        // 尝试解析为 Codeup 错误格式
        if let Ok(error) = serde_json::from_value::<CodeupErrorResponse>(data.clone()) {
            return crate::pr::codeup::errors::format_error(&error, response);
        }

        // 如果无法解析为特定格式，返回 JSON 字符串
        if let Ok(json_str) = serde_json::to_string_pretty(&data) {
            return anyhow::anyhow!(
                "API request failed: {} - {}\n\nResponse:\n{}",
                response.status,
                response.status_text,
                json_str
            );
        }
    }

    // 回退到简单错误
    anyhow::anyhow!(
        "API request failed: {} - {}",
        response.status,
        response.status_text
    )
}
