//! HTTP 响应封装
//!
//! 本模块提供了 HTTP 响应的封装和解析功能。
//! 响应体延迟解析，通过方法（as_json, as_text 等）来解析。

use color_eyre::Result;
use reqwest::header::HeaderMap;
use serde::Deserialize;

use super::parser::{JsonParser, ResponseParser, TextParser};

/// HTTP 响应格式
///
/// 封装 HTTP 响应的状态码、状态文本、响应数据和 Headers。
/// 响应体延迟解析，通过方法（as_json, as_text 等）来解析。
#[derive(Debug)]
pub struct HttpResponse {
    /// HTTP 状态码（如 200、404、500）
    pub status: u16,
    /// HTTP 状态文本（如 "OK"、"Not Found"、"Internal Server Error"）
    pub status_text: String,
    /// HTTP 响应 Headers
    pub headers: HeaderMap,
    /// 缓存的响应体字节（用于延迟解析）
    body_bytes: Vec<u8>,
}

impl HttpResponse {
    /// 从 reqwest::Response 创建 HttpResponse
    ///
    /// 只提取元数据（status、status_text、headers），并缓存响应体字节。
    /// 响应体通过后续的方法（as_json, as_text 等）来解析。
    ///
    /// # 参数
    ///
    /// * `response` - reqwest 的响应对象
    ///
    /// # 返回
    ///
    /// 返回 `HttpResponse` 结构体。
    ///
    /// # 错误
    ///
    /// 如果读取响应体失败，返回相应的错误信息。
    pub fn from_reqwest_response(response: reqwest::blocking::Response) -> Result<Self> {
        let status = response.status().as_u16();
        let status_text = response.status().canonical_reason().unwrap_or("Unknown").to_string();
        let headers = response.headers().clone();

        // 缓存响应体字节（可以多次解析）
        let body_bytes = response.bytes()?.to_vec();

        Ok(Self {
            status,
            status_text,
            headers,
            body_bytes,
        })
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

    /// 解析为 JSON（便捷方法）
    ///
    /// 将响应体解析为 JSON 并反序列化为类型 `T`。
    ///
    /// # 类型参数
    ///
    /// * `T` - 目标类型，必须实现 `Deserialize` trait
    ///
    /// # 返回
    ///
    /// 返回解析后的数据。
    ///
    /// # 错误
    ///
    /// 如果 JSON 解析失败，返回相应的错误信息。
    pub fn as_json<T>(&self) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.parse_with(JsonParser)
    }

    /// 解析为文本（便捷方法）
    ///
    /// 将响应体解析为 UTF-8 文本字符串。
    ///
    /// # 返回
    ///
    /// 返回响应体的文本内容。
    ///
    /// # 错误
    ///
    /// 如果读取响应体失败或不是有效的 UTF-8，返回相应的错误信息。
    pub fn as_text(&self) -> Result<String> {
        self.parse_with(TextParser)
    }

    /// 解析为字节
    ///
    /// 返回响应体的原始字节。
    ///
    /// # 返回
    ///
    /// 返回响应体字节的引用。
    pub fn as_bytes(&self) -> &[u8] {
        &self.body_bytes
    }

    /// 确保响应是成功的，否则返回错误
    ///
    /// 检查 HTTP 状态码是否在成功范围内（200-299）。
    /// 如果响应失败，返回包含状态码和响应体的错误信息。
    ///
    /// # 返回
    ///
    /// 如果响应成功，返回 `Ok(self)`；否则返回包含错误信息的 `Err`。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use serde_json::Value;
    /// use workflow::base::http::HttpResponse;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = workflow::base::http::HttpClient::global()?;
    /// # let url = "https://api.example.com";
    /// # let config = workflow::base::http::RequestConfig::<Value, Value>::new();
    /// # let response = client.get(url, config)?;
    /// let response = response.ensure_success()?; // 如果失败会返回错误
    /// # Ok(())
    /// # }
    /// ```
    pub fn ensure_success(self) -> Result<Self> {
        if !self.is_success() {
            color_eyre::eyre::bail!(
                "HTTP request failed with status {}: {}",
                self.status,
                self.as_text().unwrap_or_else(|_| "Unable to read response body".to_string())
            );
        }
        Ok(self)
    }

    /// 确保响应是成功的，使用自定义错误处理器
    ///
    /// 检查 HTTP 状态码是否在成功范围内（200-299）。
    /// 如果响应失败，使用提供的错误处理器生成错误。
    /// 如果响应成功，返回 `Ok(self)` 以便链式调用。
    ///
    /// # 参数
    ///
    /// * `error_handler` - 错误处理函数，接收 `&HttpResponse` 并返回错误
    ///
    /// # 返回
    ///
    /// 如果响应成功，返回 `Ok(self)`；否则返回错误处理器生成的错误。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use serde_json::Value;
    /// use workflow::base::http::HttpResponse;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = workflow::base::http::HttpClient::global()?;
    /// # let url = "https://api.example.com";
    /// # let config = workflow::base::http::RequestConfig::<Value, Value>::new();
    /// # let response = client.post(url, config)?;
    /// let response = response
    ///     .ensure_success_with(|r| color_eyre::eyre::eyre!("Error: {}", r.status))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn ensure_success_with<E>(self, error_handler: impl FnOnce(&Self) -> E) -> Result<Self>
    where
        E: Into<color_eyre::eyre::Report>,
    {
        if !self.is_success() {
            return Err(error_handler(&self).into());
        }
        Ok(self)
    }

    /// 提取错误消息（通用方法）
    ///
    /// 尝试从响应体中提取错误信息，支持多种常见的错误格式：
    /// - JSON 格式：尝试从 `error.message`、`error` 或 `message` 字段提取
    /// - 文本格式：如果无法解析为 JSON，则作为文本返回
    ///
    /// # 返回
    ///
    /// 返回提取的错误消息字符串。如果无法提取，返回格式化的 JSON 或文本内容。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use serde_json::Value;
    /// use workflow::base::http::{HttpClient, HttpResponse, RequestConfig};
    /// use workflow::log_error;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = HttpClient::global()?;
    /// # let config = RequestConfig::<Value, Value>::new();
    /// # let response = client.get("https://api.example.com", config)?;
    /// let error_msg = response.extract_error_message();
    /// log_error!("Error: {}", error_msg);
    /// # Ok(())
    /// # }
    /// ```
    pub fn extract_error_message(&self) -> String {
        // 尝试解析错误响应为 JSON，提取详细的错误信息
        match self.as_json::<serde_json::Value>() {
            Ok(error_json) => {
                // 尝试提取常见的错误字段
                let error_detail = error_json
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .or_else(|| error_json.get("error").and_then(|e| e.as_str()))
                    .or_else(|| error_json.get("message").and_then(|m| m.as_str()));

                if let Some(detail) = error_detail {
                    format!(
                        "{} (details: {})",
                        serde_json::to_string(&error_json).unwrap_or_default(),
                        detail
                    )
                } else {
                    serde_json::to_string(&error_json).unwrap_or_default()
                }
            }
            Err(_) => {
                // 如果不是 JSON，尝试作为文本解析
                self.as_text()
                    .unwrap_or_else(|_| String::from_utf8_lossy(self.as_bytes()).to_string())
            }
        }
    }

    /// 使用指定的 Parser 解析响应（通用方法）
    ///
    /// 允许使用自定义的 Parser 来解析响应体。
    ///
    /// # 类型参数
    ///
    /// * `P` - Parser 类型，必须实现 `ResponseParser<T>`
    /// * `T` - 目标类型
    ///
    /// # 参数
    ///
    /// * `parser` - 响应解析器实例
    ///
    /// # 返回
    ///
    /// 返回解析后的数据。
    fn parse_with<P, T>(&self, _parser: P) -> Result<T>
    where
        P: ResponseParser<T> + Sized,
    {
        P::parse(&self.body_bytes, self.status)
    }
}
