//! HTTP 响应封装
//!
//! 本模块提供了 HTTP 响应的封装和解析功能。
//! 响应体延迟解析，通过方法（as_json, as_text 等）来解析。

use anyhow::Result;
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
        let status_text = response
            .status()
            .canonical_reason()
            .unwrap_or("Unknown")
            .to_string();
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
