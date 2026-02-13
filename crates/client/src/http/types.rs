//! HTTP 请求/响应数据结构（定义层）
//!
//! 纯数据结构，使用标准库类型，不依赖 reqwest。

use std::collections::HashMap;
use std::time::Duration;

use crate::{ErrorContext, HttpError};

use super::{Authorization, HttpMethod};

/// HTTP 请求（纯数据）
///
/// 定义层类型，实现层负责将其转换为具体 HTTP 库的请求。
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// 请求方法
    pub method: HttpMethod,
    /// 请求 URL
    pub url: String,
    /// 请求头
    pub headers: HashMap<String, String>,
    /// 查询参数（用于序列化为 query string）
    pub query: Option<serde_json::Value>,
    /// 请求体（JSON）
    pub body: Option<serde_json::Value>,
    /// 认证信息
    pub auth: Option<Authorization>,
    /// 超时时间
    pub timeout: Option<Duration>,
}

impl HttpRequest {
    /// 创建 GET 请求
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::GET,
            url: url.into(),
            headers: HashMap::new(),
            query: None,
            body: None,
            auth: None,
            timeout: None,
        }
    }

    /// 创建 POST 请求
    pub fn post(url: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::POST,
            url: url.into(),
            headers: HashMap::new(),
            query: None,
            body: None,
            auth: None,
            timeout: None,
        }
    }

    /// 创建 PUT 请求
    pub fn put(url: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::PUT,
            url: url.into(),
            headers: HashMap::new(),
            query: None,
            body: None,
            auth: None,
            timeout: None,
        }
    }

    /// 创建 DELETE 请求
    pub fn delete(url: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::DELETE,
            url: url.into(),
            headers: HashMap::new(),
            query: None,
            body: None,
            auth: None,
            timeout: None,
        }
    }

    /// 创建 PATCH 请求
    pub fn patch(url: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::PATCH,
            url: url.into(),
            headers: HashMap::new(),
            query: None,
            body: None,
            auth: None,
            timeout: None,
        }
    }
}

/// HTTP 响应（纯数据）
///
/// 定义层类型，实现层负责从具体 HTTP 库的响应构造。
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// 状态码
    pub status: u16,
    /// 响应头
    pub headers: HashMap<String, String>,
    /// 响应体
    pub body: Vec<u8>,
    /// 请求 URL
    pub url: String,
    /// 请求方法
    pub method: HttpMethod,
    /// 请求耗时
    pub duration: Duration,
}

impl HttpResponse {
    /// 是否是成功响应（2xx）
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// 是否是客户端错误（4xx）
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// 是否是服务端错误（5xx）
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }

    /// 获取响应体字节
    pub fn bytes(&self) -> &[u8] {
        &self.body
    }

    /// 尝试将响应体解析为 UTF-8 文本
    pub fn text(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.body)
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, HttpError> {
        serde_json::from_slice(&self.body).map_err(|e| HttpError::ResponseParse {
            message: "Failed to parse response as JSON".to_string(),
            context: ErrorContext::new(&self.url, self.method)
                .with_response_status(self.status)
                .with_response_headers(self.headers.clone())
                .with_error(e)
                .into_box(),
        })
    }

    /// 获取指定名称的响应头
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name.to_lowercase().as_str()).map(|s| s.as_str())
    }

    /// 提取错误消息
    ///
    /// 尝试从响应体中提取错误消息，支持常见格式：
    /// - `{"error": "message"}`
    /// - `{"error": {"message": "..."}}`
    /// - `{"message": "..."}`
    /// - `{"errors": [...]}`
    pub fn get_error_message(&self) -> Result<String, HttpError> {
        // 解析响应体为文本
        let text = self.text().map_err(|e| HttpError::ResponseParse {
            message: format!("Failed to parse response as text: {}", e),
            context: ErrorContext::new(&self.url, self.method)
                .with_response_status(self.status)
                .with_response_headers(self.headers.clone())
                .into_box(),
        })?;

        // 解析响应体为 JSON
        let json = serde_json::from_str::<serde_json::Value>(text).map_err(|e| {
            HttpError::ResponseParse {
                message: format!("Failed to parse response as JSON: {}", e),
                context: ErrorContext::new(&self.url, self.method)
                    .with_response_status(self.status)
                    .with_response_headers(self.headers.clone())
                    .into_box(),
            }
        })?;

        if let Some(msg) = json.get("error") {
            if let Some(s) = msg.as_str() {
                return Ok(s.to_string());
            }
            if let Some(inner) = msg.get("message").and_then(|m| m.as_str()) {
                return Ok(inner.to_string());
            }
        }

        if let Some(msg) = json.get("message").and_then(|m| m.as_str()) {
            return Ok(msg.to_string());
        }

        if let Some(errors) = json.get("errors").and_then(|e| e.as_array()) {
            if let Some(first) = errors.first() {
                if let Some(msg) = first.get("message").and_then(|m| m.as_str()) {
                    return Ok(msg.to_string());
                }
                if let Some(s) = first.as_str() {
                    return Ok(s.to_string());
                }
            }
        }

        if !text.is_empty() {
            return Ok(text.to_string());
        }

        // 返回整个 JSON
        Err(HttpError::ResponseParse {
            message: "No error message found".to_string(),
            context: ErrorContext::new(&self.url, self.method).into_box(),
        })
    }
}
