use anyhow::Context;
use reqwest::header::HeaderMap;
use serde::Deserialize;

/// HTTP 响应格式
#[derive(Debug)]
pub struct HttpResponse<T> {
    pub status: u16,
    pub status_text: String,
    pub data: T,
    pub headers: HeaderMap,
}

impl<T> HttpResponse<T> {
    /// 创建新的 HttpResponse
    pub fn new(status: u16, status_text: String, data: T, headers: HeaderMap) -> Self {
        Self {
            status,
            status_text,
            data,
            headers,
        }
    }

    /// 检查是否为成功响应（状态码 200-299）
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// 检查是否为错误响应
    pub fn is_error(&self) -> bool {
        !self.is_success()
    }
}

/// 从 reqwest::Response 转换为 HttpResponse
impl<T> HttpResponse<T>
where
    T: for<'de> Deserialize<'de>,
{
    pub fn from_reqwest_response(
        response: reqwest::blocking::Response,
    ) -> Result<Self, anyhow::Error> {
        let status = response.status().as_u16();
        let status_text = response
            .status()
            .canonical_reason()
            .unwrap_or("Unknown")
            .to_string();
        let headers = response.headers().clone();

        // 先读取响应体文本（某些 API 如 Jira transitions 可能返回空响应体）
        let text = response.text()?;

        // 如果响应体为空，尝试解析为 null JSON（适用于 serde_json::Value）
        let data: T = if text.trim().is_empty() {
            // 空响应体，尝试解析为 null
            serde_json::from_str("null")
                .or_else(|_| serde_json::from_str("{}"))
                .context("Failed to parse empty response as JSON")?
        } else {
            // 非空响应体，正常解析 JSON
            serde_json::from_str(&text)
                .context(format!("Failed to parse JSON response: {}", text))?
        };

        Ok(Self {
            status,
            status_text,
            data,
            headers,
        })
    }
}
