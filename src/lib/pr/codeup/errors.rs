//! Codeup 错误处理模块
//!
//! 提供 Codeup API 特定的错误响应结构和格式化函数

use crate::base::http::HttpResponse;
use anyhow::Error;
use serde::Deserialize;
use serde_json::Value;

/// Codeup 错误响应结构
#[derive(Debug, Deserialize)]
pub struct CodeupErrorResponse {
    pub message: Option<String>,
    // TODO: 根据 Codeup API 实际错误格式添加字段
}

/// 格式化 Codeup 错误信息
///
/// 将 Codeup API 错误响应格式化为用户友好的错误消息
pub fn format_error(error: &CodeupErrorResponse, response: &HttpResponse) -> Error {
    // TODO: 实现更详细的 Codeup 错误格式化
    let mut msg = format!(
        "Codeup API request failed: {} - {}",
        response.status, response.status_text
    );

    if let Some(message) = &error.message {
        msg.push_str(&format!("\nMessage: {}", message));
    }

    // 尝试添加完整的错误响应 JSON 以便调试
    if let Ok(data) = response.as_json::<Value>() {
        if let Ok(json_str) = serde_json::to_string_pretty(&data) {
            msg.push_str(&format!("\n\nFull error response:\n{}", json_str));
        }
    }

    anyhow::anyhow!(msg)
}
