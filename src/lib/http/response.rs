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

        let data: T = response.json()?;

        Ok(Self {
            status,
            status_text,
            data,
            headers,
        })
    }
}
