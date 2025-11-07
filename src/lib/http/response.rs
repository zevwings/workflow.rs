//! HTTP 响应封装
//!
//! 本模块提供了 HTTP 响应的封装和解析功能。

use anyhow::Context;
use reqwest::header::HeaderMap;
use serde::Deserialize;

/// HTTP 响应格式
///
/// 封装 HTTP 响应的状态码、状态文本、响应数据和 Headers。
#[derive(Debug)]
pub struct HttpResponse<T> {
    /// HTTP 状态码（如 200、404、500）
    pub status: u16,
    /// HTTP 状态文本（如 "OK"、"Not Found"、"Internal Server Error"）
    pub status_text: String,
    /// 响应数据（已解析为类型 `T`）
    pub data: T,
    /// HTTP 响应 Headers
    pub headers: HeaderMap,
}

impl<T> HttpResponse<T> {
    /// 创建新的 HttpResponse
    ///
    /// 手动创建 HTTP 响应结构体。
    ///
    /// # 参数
    ///
    /// * `status` - HTTP 状态码
    /// * `status_text` - HTTP 状态文本
    /// * `data` - 响应数据
    /// * `headers` - HTTP 响应 Headers
    ///
    /// # 返回
    ///
    /// 返回 `HttpResponse<T>` 结构体。
    pub fn new(status: u16, status_text: String, data: T, headers: HeaderMap) -> Self {
        Self {
            status,
            status_text,
            data,
            headers,
        }
    }

    /// 检查是否为成功响应（状态码 200-299）
    ///
    /// 判断 HTTP 状态码是否在成功范围内（200-299）。
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果状态码在 200-299 范围内，否则返回 `false`。
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// 检查是否为错误响应
    ///
    /// 判断 HTTP 状态码是否不在成功范围内（即状态码 < 200 或 >= 300）。
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果状态码不在 200-299 范围内，否则返回 `false`。
    pub fn is_error(&self) -> bool {
        !self.is_success()
    }
}

/// 从 reqwest::Response 转换为 HttpResponse
impl<T> HttpResponse<T>
where
    T: for<'de> Deserialize<'de>,
{
    /// 从 reqwest::Response 转换为 HttpResponse
    ///
    /// 将 `reqwest::blocking::Response` 转换为 `HttpResponse<T>`。
    /// 响应体会被解析为 JSON 并反序列化为类型 `T`。
    ///
    /// # 参数
    ///
    /// * `response` - reqwest 的响应对象
    ///
    /// # 返回
    ///
    /// 返回 `HttpResponse<T>` 结构体，包含状态码、状态文本、解析后的数据和 Headers。
    ///
    /// # 错误
    ///
    /// 如果读取响应体失败或 JSON 解析失败，返回相应的错误信息。
    /// 对于空响应体，会尝试解析为 `null` 或 `{}`。
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

        // 先读取响应体字节（比 text() 更高效，避免额外的 UTF-8 验证）
        let bytes = response.bytes()?;

        // 如果响应体为空，尝试解析为 null JSON（适用于 serde_json::Value）
        let data: T = if bytes.is_empty() || bytes.iter().all(|&b| b.is_ascii_whitespace()) {
            // 空响应体，尝试解析为 null
            serde_json::from_slice(b"null")
                .or_else(|_| serde_json::from_slice(b"{}"))
                .context("Failed to parse empty response as JSON")?
        } else {
            // 非空响应体，尝试解析 JSON
            serde_json::from_slice(&bytes).with_context(|| {
                // 如果解析失败，尝试读取响应体内容用于错误信息
                let response_text = String::from_utf8_lossy(&bytes);
                let preview = if response_text.len() > 200 {
                    format!("{}...", &response_text[..200])
                } else {
                    response_text.to_string()
                };
                format!(
                    "Failed to parse JSON response (HTTP {}). The response may be HTML or an error page. Response preview: {}",
                    status, preview
                )
            })?
        };

        Ok(Self {
            status,
            status_text,
            data,
            headers,
        })
    }
}
