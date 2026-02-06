//! HTTP 响应封装

use std::collections::HashMap;
use std::time::Duration;

use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;

use super::error::{ErrorContext, HttpError};
use super::method::HttpMethod;

/// HTTP 响应
#[derive(Debug)]
pub struct Response {
    /// 状态码
    pub status: u16,
    /// 响应头
    pub headers: HeaderMap,
    /// 响应体
    body: Vec<u8>,
    /// 请求 URL
    url: String,
    /// 请求方法
    method: HttpMethod,
    /// 请求耗时
    duration: Duration,
}

impl Response {
    /// 从 reqwest 响应创建
    pub(crate) fn from_reqwest(
        response: reqwest::blocking::Response,
        method: HttpMethod,
        duration: Duration,
        max_body_size: usize,
    ) -> Result<Self, HttpError> {
        let status = response.status().as_u16();
        let headers = response.headers().clone();
        let url = response.url().to_string();

        // 读取响应体
        let body = response
            .bytes()
            .map_err(|e| HttpError::ResponseParse {
                message: format!("Failed to read response body: {}", e),
                context: ErrorContext::new(&url, method)
                    .with_response_status(status)
                    .with_response_headers(&headers)
                    .into_box(),
            })?
            .to_vec();

        // 检查响应体大小
        if body.len() > max_body_size {
            return Err(HttpError::ResponseParse {
                message: format!(
                    "Response body too large: {} bytes (max: {} bytes)",
                    body.len(),
                    max_body_size
                ),
                context: ErrorContext::new(&url, method)
                    .with_response_status(status)
                    .with_response_headers(&headers)
                    .into_box(),
            });
        }

        Ok(Self {
            status,
            headers,
            body,
            url,
            method,
            duration,
        })
    }

    /// 获取请求 URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// 获取请求方法
    pub fn method(&self) -> HttpMethod {
        self.method
    }

    /// 获取请求耗时
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// 是否是成功响应（2xx）
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// 是否是错误响应
    pub fn is_error(&self) -> bool {
        !self.is_success()
    }

    /// 是否是客户端错误（4xx）
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// 是否是服务端错误（5xx）
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }

    /// 解析为 JSON
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, HttpError> {
        serde_json::from_slice(&self.body).map_err(|e| HttpError::ResponseParse {
            message: format!("Failed to parse JSON: {}", e),
            context: self.error_context_boxed(),
        })
    }

    /// 解析为文本（借用）
    pub fn text(&self) -> Result<&str, HttpError> {
        std::str::from_utf8(&self.body).map_err(|e| HttpError::ResponseParse {
            message: format!("Failed to parse text: {}", e),
            context: self.error_context_boxed(),
        })
    }

    /// 解析为文本（消费 self，避免克隆）
    pub fn into_text(self) -> Result<String, HttpError> {
        String::from_utf8(self.body).map_err(|e| HttpError::ResponseParse {
            message: format!("Failed to parse text: {}", e.utf8_error()),
            context: ErrorContext::new(&self.url, self.method)
                .with_response_status(self.status)
                .with_response_headers(&self.headers)
                .with_duration(self.duration)
                .into_box(),
        })
    }

    /// 获取原始字节
    pub fn bytes(&self) -> &[u8] {
        &self.body
    }

    /// 获取响应头
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }

    /// 获取所有响应头（转换为 HashMap）
    pub fn headers_map(&self) -> HashMap<String, String> {
        self.headers
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.as_str().to_string(), v.to_string())))
            .collect()
    }

    /// 确保响应成功，否则返回错误
    pub fn ensure_success(self) -> Result<Self, HttpError> {
        if self.is_success() {
            Ok(self)
        } else {
            Err(HttpError::Status {
                status: self.status,
                context: self.error_context_boxed(),
            })
        }
    }

    /// 使用自定义检查确保响应成功
    pub fn ensure_success_with<F>(self, check: F) -> Result<Self, HttpError>
    where
        F: FnOnce(&Self) -> bool,
    {
        if check(&self) {
            Ok(self)
        } else {
            Err(HttpError::Status {
                status: self.status,
                context: self.error_context_boxed(),
            })
        }
    }

    /// 提取错误消息
    ///
    /// 尝试从响应体中提取错误消息，支持常见格式：
    /// - `{"error": "message"}`
    /// - `{"error": {"message": "..."}}`
    /// - `{"message": "..."}`
    /// - `{"errors": [...]}`
    pub fn extract_error_message(&self) -> String {
        if let Ok(text) = self.text() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                // 尝试各种常见格式
                if let Some(msg) = json.get("error") {
                    if let Some(s) = msg.as_str() {
                        return s.to_string();
                    }
                    if let Some(inner) = msg.get("message").and_then(|m| m.as_str()) {
                        return inner.to_string();
                    }
                }
                if let Some(msg) = json.get("message").and_then(|m| m.as_str()) {
                    return msg.to_string();
                }
                if let Some(errors) = json.get("errors").and_then(|e| e.as_array()) {
                    if let Some(first) = errors.first() {
                        if let Some(msg) = first.get("message").and_then(|m| m.as_str()) {
                            return msg.to_string();
                        }
                        if let Some(s) = first.as_str() {
                            return s.to_string();
                        }
                    }
                }
                // 返回整个 JSON
                return text.to_string();
            }
            // 返回原始文本
            return text.to_string();
        }
        format!("HTTP {}", self.status)
    }

    /// 创建错误上下文
    fn error_context(&self) -> ErrorContext {
        let body_str = String::from_utf8_lossy(&self.body).to_string();
        ErrorContext::new(&self.url, self.method)
            .with_response_status(self.status)
            .with_response_headers(&self.headers)
            .with_response_body(body_str)
            .with_duration(self.duration)
    }

    /// 创建装箱的错误上下文
    fn error_context_boxed(&self) -> Box<ErrorContext> {
        self.error_context().into_box()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn create_test_response(status: u16, body: &[u8]) -> Response {
        Response {
            status,
            headers: HeaderMap::new(),
            body: body.to_vec(),
            url: "https://example.com/test".to_string(),
            method: HttpMethod::GET,
            duration: Duration::from_millis(100),
        }
    }

    #[test]
    fn test_is_success() {
        assert!(create_test_response(200, b"").is_success());
        assert!(create_test_response(201, b"").is_success());
        assert!(create_test_response(299, b"").is_success());
        assert!(!create_test_response(400, b"").is_success());
        assert!(!create_test_response(500, b"").is_success());
    }

    #[test]
    fn test_is_client_error() {
        assert!(create_test_response(400, b"").is_client_error());
        assert!(create_test_response(404, b"").is_client_error());
        assert!(create_test_response(499, b"").is_client_error());
        assert!(!create_test_response(200, b"").is_client_error());
        assert!(!create_test_response(500, b"").is_client_error());
    }

    #[test]
    fn test_is_server_error() {
        assert!(create_test_response(500, b"").is_server_error());
        assert!(create_test_response(502, b"").is_server_error());
        assert!(create_test_response(599, b"").is_server_error());
        assert!(!create_test_response(200, b"").is_server_error());
        assert!(!create_test_response(400, b"").is_server_error());
    }

    #[test]
    fn test_json_parsing() {
        let response = create_test_response(200, br#"{"name": "test", "value": 42}"#);
        let json: serde_json::Value = response.json().unwrap();
        assert_eq!(json["name"], "test");
        assert_eq!(json["value"], 42);
    }

    #[test]
    fn test_text_parsing() {
        let response = create_test_response(200, b"Hello, World!");
        let text = response.text().unwrap();
        assert_eq!(text, "Hello, World!");
    }

    #[test]
    fn test_ensure_success() {
        let response = create_test_response(200, b"");
        assert!(response.ensure_success().is_ok());

        let response = create_test_response(404, b"");
        assert!(response.ensure_success().is_err());
    }

    #[test]
    fn test_extract_error_message() {
        // {"error": "message"}
        let response = create_test_response(400, br#"{"error": "Something went wrong"}"#);
        assert_eq!(response.extract_error_message(), "Something went wrong");

        // {"message": "..."}
        let response = create_test_response(400, br#"{"message": "Bad request"}"#);
        assert_eq!(response.extract_error_message(), "Bad request");

        // {"error": {"message": "..."}}
        let response = create_test_response(400, br#"{"error": {"message": "Nested error"}}"#);
        assert_eq!(response.extract_error_message(), "Nested error");
    }
}
