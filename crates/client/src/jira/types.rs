//! Jira 客户端响应类型

use crate::{jira::JiraClientError, HttpError, HttpResponse};

/// Jira API 响应包装器
///
/// 封装 HTTP 响应，提供延迟解析能力。
/// 与 `GitHubResponse` 保持一致的设计模式。
#[derive(Debug)]
pub struct JiraResponse {
    response: HttpResponse,
}

impl JiraResponse {
    /// 创建响应包装器
    pub fn new(response: HttpResponse) -> Self {
        Self { response }
    }

    /// 获取 HTTP 状态码
    pub fn status(&self) -> u16 {
        self.response.status
    }

    /// 检查是否是成功响应
    pub fn is_success(&self) -> bool {
        self.response.is_success()
    }

    /// 解析为 JSON（便捷方法）
    ///
    /// 将响应体解析为 JSON 并反序列化为类型 `T`。
    pub fn json<T>(&self) -> Result<T, HttpError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        self.response.json()
    }

    /// 解析为特定模型类型（便捷方法）
    ///
    /// 与 `json()` 类似，但返回 `JiraClientError`
    pub fn as_model<T>(&self) -> Result<T, JiraClientError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        self.response
            .json()
            .map_err(|e| JiraClientError::ApiError(format!("Failed to parse JSON response: {}", e)))
    }

    /// 解析为文本（便捷方法）
    pub fn text(&self) -> Result<&str, JiraClientError> {
        self.response.text().map_err(|e| JiraClientError::ApiError(e.to_string()))
    }

    /// 获取响应体字节
    pub fn bytes(&self) -> &[u8] {
        self.response.bytes()
    }
}
