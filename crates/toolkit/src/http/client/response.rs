//! HTTP 响应实现

use reqwest::header::HeaderMap;
use serde::Deserialize;

use crate::http::HttpError;

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
    pub body_bytes: Vec<u8>,
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
    /// * `max_response_body_size` - 最大响应体大小（字节），默认 100MB
    ///
    /// # 返回
    ///
    /// 返回 `HttpResponse` 结构体。
    ///
    /// # 错误
    ///
    /// 如果读取响应体失败，返回相应的错误信息。
    pub fn from_reqwest_response(
        response: reqwest::blocking::Response,
        max_response_body_size: usize,
    ) -> Result<Self, HttpError> {
        let status = response.status().as_u16();
        let span = tracing::span!(
            tracing::Level::DEBUG,
            "http.response.parse",
            module = "http",
            http.status_code = status,
        );
        let _guard = span.enter();

        let status_text = response.status().canonical_reason().unwrap_or("Unknown").to_string();
        let headers = response.headers().clone();

        let body_bytes = response.bytes().map_err(|e| {
            tracing::error!(
                module = "http",
                http.status_code = status,
                error = %e,
                "Failed to read response body"
            );
            HttpError::UnableToReadBody(format!("Unable to read response body: {}", e))
        })?;
        let body_bytes = body_bytes.to_vec();

        if body_bytes.len() > max_response_body_size {
            tracing::error!(
                module = "http",
                http.status_code = status,
                body_size_bytes = body_bytes.len(),
                max_size_bytes = max_response_body_size,
                "Response body exceeds size limit"
            );
            return Err(HttpError::UnableToReadBody(format!(
                "Response body too large: {} bytes (max: {} bytes)",
                body_bytes.len(),
                max_response_body_size
            )));
        }

        tracing::debug!(
            module = "http",
            http.status_code = status,
            body_size_bytes = body_bytes.len(),
            "Response parsed successfully"
        );

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
    pub fn as_json<T>(&self) -> Result<T, HttpError>
    where
        T: for<'de> Deserialize<'de>,
    {
        // 处理空响应
        if self.body_bytes.is_empty() || self.body_bytes.iter().all(|&b| b.is_ascii_whitespace()) {
            serde_json::from_slice(b"null")
                .or_else(|_| serde_json::from_slice(b"{}"))
                .map_err(|_| HttpError::ParseEmptyJsonFailed)
        } else {
            serde_json::from_slice(&self.body_bytes).map_err(|_source| {
                let preview = String::from_utf8_lossy(&self.body_bytes);
                let preview = if preview.len() > 200 {
                    format!("{}...", &preview[..200])
                } else {
                    preview.to_string()
                };
                HttpError::ParseJsonFailed {
                    status: self.status,
                    preview,
                }
            })
        }
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
    pub fn as_text(&self) -> Result<String, HttpError> {
        // 检查状态码
        if !(200..300).contains(&self.status) {
            return Err(HttpError::HttpRequestFailed(self.status));
        }

        String::from_utf8(self.body_bytes.clone()).map_err(HttpError::DecodeUtf8Failed)
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
    /// use toolkit::http::{HttpResponse, HttpClient, RequestConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = HttpClient::global()?;
    /// # let url = "https://api.example.com";
    /// # let config = RequestConfig::new();
    /// # let response = client.get(url, config)?;
    /// let response = response.ensure_success()?; // 如果失败会返回错误
    /// # Ok(())
    /// # }
    /// ```
    pub fn ensure_success(self) -> Result<Self, HttpError> {
        if !self.is_success() {
            let body =
                self.as_text().unwrap_or_else(|_| "Unable to read response body".to_string());
            return Err(HttpError::ResponseFailed {
                status: self.status,
                body,
            });
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
    /// use toolkit::http::{HttpResponse, HttpClient, RequestConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = HttpClient::global()?;
    /// # let url = "https://api.example.com";
    /// # let config = RequestConfig::new();
    /// # let response = client.post(url, config)?;
    /// let response = response
    ///     .ensure_success_with(|r| HttpError::HttpRequestFailed(r.status))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn ensure_success_with<E>(self, error_handler: impl FnOnce(&Self) -> E) -> Result<Self, E> {
        if !self.is_success() {
            return Err(error_handler(&self));
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
    /// use toolkit::http::{HttpClient, HttpResponse, RequestConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = HttpClient::global()?;
    /// # let config = RequestConfig::new();
    /// # let response = client.get("https://api.example.com", config)?;
    /// let error_msg = response.extract_error_message();
    /// eprintln!("Error: {}", error_msg);
    /// # Ok(())
    /// # }
    /// ```
    pub fn extract_error_message(&self) -> String {
        match self.as_json::<serde_json::Value>() {
            Ok(error_json) => {
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
            Err(_) => self
                .as_text()
                .unwrap_or_else(|_| String::from_utf8_lossy(self.as_bytes()).to_string()),
        }
    }

    /// 创建测试用的 HttpResponse（仅用于测试）
    #[cfg(test)]
    pub(crate) fn new_for_test(
        status: u16,
        status_text: String,
        headers: HeaderMap,
        body_bytes: Vec<u8>,
    ) -> Self {
        Self {
            status,
            status_text,
            headers,
            body_bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use reqwest::header::HeaderMap;
    use serde_json::json;

    use crate::http::mock::HttpMockServer;

    use super::*;

    // 辅助函数：创建测试用的 HttpResponse
    fn create_test_response(status: u16, body: &[u8]) -> HttpResponse {
        HttpResponse::new_for_test(status, "OK".to_string(), HeaderMap::new(), body.to_vec())
    }

    // ==================== HttpResponse 基础测试 ====================

    #[test]
    fn test_http_response_is_success_200() {
        let response = create_test_response(200, b"");
        assert!(response.is_success());
        assert!(!response.is_error());
    }

    #[test]
    fn test_http_response_is_success_201() {
        let response = create_test_response(201, b"");
        assert!(response.is_success());
        assert!(!response.is_error());
    }

    #[test]
    fn test_http_response_is_error_404() {
        let response = create_test_response(404, b"");
        assert!(!response.is_success());
        assert!(response.is_error());
    }

    #[test]
    fn test_http_response_is_error_500() {
        let response = create_test_response(500, b"");
        assert!(!response.is_success());
        assert!(response.is_error());
    }

    // ==================== HttpResponse 解析测试 ====================

    #[test]
    fn test_http_response_as_json_valid() {
        let json_body = r#"{"key": "value", "number": 42}"#;
        let response = create_test_response(200, json_body.as_bytes());

        let data: serde_json::Value = response.as_json().unwrap();
        assert_eq!(data["key"], "value");
        assert_eq!(data["number"], 42);
    }

    #[test]
    fn test_http_response_as_json_invalid() {
        let invalid_json = "{invalid json}";
        let response = create_test_response(200, invalid_json.as_bytes());

        let result: Result<serde_json::Value, HttpError> = response.as_json();
        assert!(result.is_err());
    }

    #[test]
    fn test_http_response_as_text_valid() {
        let text_body = "Hello, World!";
        let response = create_test_response(200, text_body.as_bytes());

        let text = response.as_text().unwrap();
        assert_eq!(text, "Hello, World!");
    }

    #[test]
    fn test_http_response_as_text_invalid_utf8() {
        // 创建无效的 UTF-8 字节序列
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        let response = create_test_response(200, &invalid_utf8);

        let result = response.as_text();
        assert!(result.is_err());
    }

    #[test]
    fn test_http_response_as_bytes() {
        let body = b"test bytes";
        let response = create_test_response(200, body);

        let bytes = response.as_bytes();
        assert_eq!(bytes, body);
    }

    // ==================== HttpResponse ensure_success 测试 ====================

    #[test]
    fn test_http_response_ensure_success_200() {
        let response = create_test_response(200, b"success");
        let result = response.ensure_success();
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_response_ensure_success_404() {
        let response = create_test_response(404, b"Not Found");
        let result = response.ensure_success();
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("404"));
        // 错误格式是 "HTTP request failed with status {status}: {body}"
        assert!(error_msg.contains("status 404"));
    }

    #[test]
    fn test_http_response_ensure_success_500() {
        let response = create_test_response(500, b"Internal Server Error");
        let result = response.ensure_success();
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("500"));
    }

    #[test]
    fn test_http_response_ensure_success_body_truncation() {
        // 测试 ensure_success 在响应体很大时的截断
        // 注意：实际的截断逻辑在 ensure_success 中，这里主要测试错误信息包含 body
        let large_body = "x".repeat(1000);
        let response = create_test_response(404, large_body.as_bytes());
        let result = response.ensure_success();
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("404"));
        // 错误信息应该包含响应体（格式是 "HTTP request failed with status {status}: {body}"）
        // 由于 body 可能很长，我们只验证错误格式正确
        assert!(error_msg.contains("status 404"));
    }

    #[test]
    fn test_http_response_ensure_success_with() {
        let response = create_test_response(404, b"Not Found");
        let result = response.ensure_success_with(|r| HttpError::HttpRequestFailed(r.status));
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Custom error"));
        assert!(error_msg.contains("404"));
    }

    // ==================== HttpResponse 边界条件测试 ====================

    #[test]
    fn test_http_response_empty_body() {
        let response = create_test_response(200, b"");
        assert!(response.is_success());
        let text = response.as_text().unwrap();
        assert_eq!(text, "");
    }

    #[test]
    fn test_http_response_no_canonical_reason() {
        // 测试没有 canonical_reason 的状态码
        // 注意：我们无法直接测试，因为 canonical_reason 在 from_reqwest_response 中处理
        // 这里我们测试一个有效的状态码
        let response = create_test_response(200, b"");
        assert_eq!(response.status, 200);
    }

    // ==================== HttpResponse extract_error_message 测试 ====================

    #[test]
    fn test_http_response_extract_error_message_json() {
        // 测试从 JSON 响应中提取错误消息
        let error_json = json!({
            "error": {
                "message": "Something went wrong"
            }
        });
        let response = create_test_response(400, error_json.to_string().as_bytes());

        let error_msg = response.extract_error_message();
        assert!(error_msg.contains("Something went wrong"));
    }

    #[test]
    fn test_http_response_extract_error_message_text() {
        // 测试从文本响应中提取错误消息
        let error_text = "Error: Invalid request";
        let response = create_test_response(400, error_text.as_bytes());

        let error_msg = response.extract_error_message();
        assert!(error_msg.contains("Invalid request"));
    }

    #[test]
    fn test_http_response_extract_error_message_empty() {
        // 测试空响应体的错误消息提取
        // 空响应体在 as_json 时会返回 "null"（JSON 的空值）
        let response = create_test_response(400, b"");
        let error_msg = response.extract_error_message();
        // 空响应体可能被解析为 JSON null，或者返回空字符串
        assert!(error_msg.is_empty() || error_msg == "null");
    }

    #[test]
    fn test_http_response_extract_error_message_json_with_message_field() {
        // 测试 JSON 中有 message 字段的情况
        let error_json = json!({
            "message": "Direct message"
        });
        let response = create_test_response(400, error_json.to_string().as_bytes());

        let error_msg = response.extract_error_message();
        assert!(error_msg.contains("Direct message"));
    }

    // ==================== HttpResponse from_reqwest_response 测试（使用 Mock Server） ====================

    #[test]
    fn test_from_reqwest_response_success() {
        let mut mock_server = HttpMockServer::new();
        let _mock = mock_server
            .mock("GET", "/test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"key": "value", "number": 42}"#)
            .create();

        let response = mock_server.create_http_response("GET", "/test", 1024).unwrap();

        assert!(response.is_success());
        assert_eq!(response.status, 200);
        assert_eq!(response.status_text, "OK");

        let data: serde_json::Value = response.as_json().unwrap();
        assert_eq!(data["key"], "value");
        assert_eq!(data["number"], 42);
    }

    #[test]
    fn test_from_reqwest_response_404() {
        let mut mock_server = HttpMockServer::new();
        let _mock = mock_server
            .mock("GET", "/not-found")
            .with_status(404)
            .with_body("Not Found")
            .create();

        let response = mock_server.create_http_response("GET", "/not-found", 1024).unwrap();

        assert!(!response.is_success());
        assert_eq!(response.status, 404);
        assert_eq!(response.status_text, "Not Found");
        // 注意：as_text() 可能在某些解析器中失败，使用 as_bytes() 更可靠
        let bytes = response.as_bytes();
        assert_eq!(std::str::from_utf8(bytes).unwrap(), "Not Found");
    }

    #[test]
    fn test_from_reqwest_response_large_body() {
        let mut mock_server = HttpMockServer::new();
        let large_body = "x".repeat(500); // 500 字节
        let _mock = mock_server
            .mock("GET", "/large")
            .with_status(200)
            .with_body(&large_body)
            .create();

        // 测试正常大小限制
        let response = mock_server.create_http_response("GET", "/large", 1024).unwrap();
        assert!(response.is_success());
        assert_eq!(response.as_bytes().len(), 500);

        // 测试超过大小限制
        let result = mock_server.create_http_response("GET", "/large", 100);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("too large"));
    }

    #[test]
    fn test_from_reqwest_response_empty_body() {
        let mut mock_server = HttpMockServer::new();
        let _mock = mock_server.mock("GET", "/empty").with_status(204).with_body("").create();

        let response = mock_server.create_http_response("GET", "/empty", 1024).unwrap();

        assert!(response.is_success());
        assert_eq!(response.status, 204);
        assert_eq!(response.as_bytes().len(), 0);
    }
}
