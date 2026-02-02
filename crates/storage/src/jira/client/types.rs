//! Jira 客户端响应类型

use domain::JiraError;

/// Jira API 响应包装器
///
/// 用于统一处理 Jira API 的响应，支持泛型类型解析。
#[derive(Debug)]
pub struct JiraResponse {
    /// 解析后的响应数据
    pub data: serde_json::Value,
}

impl JiraResponse {
    /// 创建响应包装器
    pub fn new(data: serde_json::Value) -> Self {
        Self { data }
    }
}

pub trait JiraResponseSerializable {
    fn as_model<T>(&self) -> Result<T, JiraError>
    where
        T: for<'de> serde::Deserialize<'de>;
}

impl JiraResponseSerializable for JiraResponse {
    fn as_model<T>(&self) -> Result<T, JiraError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        serde_json::from_value(self.data.clone())
            .map_err(|e| JiraError::ApiError(format!("Failed to parse JSON response: {}", e)))
    }
}
