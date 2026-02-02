//! CNB API 响应处理

use domain::CNBError;
use serde::Deserialize;
use toolkit::HttpResponse;

/// CNB API 响应封装
pub struct CNBResponse {
    inner: HttpResponse,
}

impl CNBResponse {
    pub fn new(response: HttpResponse) -> Self {
        Self { inner: response }
    }

    /// 获取响应体文本
    pub fn text(&self) -> Result<String, CNBError> {
        self.inner
            .as_text()
            .map_err(|e| CNBError::Other(format!("Failed to get response text: {}", e)))
    }

    /// 将响应体解析为 JSON
    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, CNBError> {
        let text = self.text()?;
        serde_json::from_str(&text)
            .map_err(|e| CNBError::Other(format!("Failed to parse JSON: {}", e)))
    }
}

/// CNB API 错误响应
#[derive(Debug, Deserialize)]
pub struct CNBErrorResponse {
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub errcode: Option<i32>,
}
