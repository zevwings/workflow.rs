//! CNB API 客户端
//!
//! 封装 CNB API 的公共 HTTP 请求逻辑，包括：
//! - 请求头构建
//! - 错误处理
//! - API 基础 URL 管理

use std::sync::Arc;

use domain::{CNBContext, CNBError};
use serde_json::Value;
use toolkit::{HttpClient, HttpResponse, RequestConfig};

use crate::cnb::client::response::{CNBErrorResponse, CNBResponse};

pub const API_BASE: &str = "https://api.cnb.cool";

pub trait CNBClient: Send + Sync {
    /// GET 请求，返回 HTTP 响应
    ///
    /// `path` 应该是相对路径（如 `/repos/owner/project`），会自动与 API_BASE 拼接
    /// 如果传入完整 URL（以 http:// 或 https:// 开头），则直接使用
    fn get(&self, path: &str) -> Result<CNBResponse, CNBError>;

    /// POST 请求，返回 HTTP 响应
    ///
    /// `path` 应该是相对路径（如 `/repos/owner/project`），会自动与 API_BASE 拼接
    /// 如果传入完整 URL（以 http:// 或 https:// 开头），则直接使用
    fn post(&self, path: &str, body: &Value) -> Result<CNBResponse, CNBError>;

    /// PUT 请求，返回 HTTP 响应
    ///
    /// `path` 应该是相对路径（如 `/repos/owner/project`），会自动与 API_BASE 拼接
    /// 如果传入完整 URL（以 http:// 或 https:// 开头），则直接使用
    fn put(&self, path: &str, body: &Value) -> Result<CNBResponse, CNBError>;

    /// PATCH 请求，返回 HTTP 响应
    ///
    /// `path` 应该是相对路径（如 `/repos/owner/project`），会自动与 API_BASE 拼接
    /// 如果传入完整 URL（以 http:// 或 https:// 开头），则直接使用
    fn patch(&self, path: &str, body: &Value) -> Result<CNBResponse, CNBError>;
}

/// CNB API 客户端实现
pub struct CNBClientImpl {
    context: Arc<dyn CNBContext>,
}

impl CNBClientImpl {
    pub fn new(context: Arc<dyn CNBContext>) -> Self {
        Self { context }
    }

    /// 构建完整的 URL
    ///
    /// 如果传入的路径已经是完整 URL（以 http:// 或 https:// 开头），则直接返回
    /// 否则将相对路径与 API_BASE 拼接
    fn build_url(&self, path: &str) -> Result<String, CNBError> {
        if path.starts_with("http://") || path.starts_with("https://") {
            return Ok(path.to_string());
        }

        Ok(format!("{}{}", API_BASE, path))
    }

    /// 构建请求头
    fn build_headers(&self) -> Result<reqwest::header::HeaderMap, CNBError> {
        let token = self.context.get_api_token()?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().map_err(|e| {
                CNBError::Other(format!("Failed to parse Authorization header: {}", e))
            })?,
        );
        headers.insert(
            "Accept",
            "application/json"
                .parse()
                .map_err(|e| CNBError::Other(format!("Failed to parse Accept header: {}", e)))?,
        );
        headers.insert(
            "User-Agent",
            "Workflow-CLI".parse().map_err(|e| {
                CNBError::Other(format!("Failed to parse User-Agent header: {}", e))
            })?,
        );

        Ok(headers)
    }

    /// 处理 HTTP 响应
    fn handle_response(&self, response: HttpResponse, url: &str, request_body: Option<&str>) -> Result<CNBResponse, CNBError> {
        let status = response.status;

        // 调试输出 - 打印响应信息（使用 body_bytes 避免 as_text 对状态码的检查）
        let body_text = String::from_utf8_lossy(&response.body_bytes).to_string();
        toolkit::log_debug!("CNB API Response: Status={}, Body={}", status, body_text);

        // 2xx 成功
        if (200..300).contains(&status) {
            return Ok(CNBResponse::new(response));
        }

        // 尝试解析为 JSON 错误
        if let Ok(error_response) = serde_json::from_str::<CNBErrorResponse>(&body_text) {
            let error_msg = if !error_response.message.is_empty() {
                error_response.message.clone()
            } else if let Some(errcode) = error_response.errcode {
                format!("CNB API error code: {}", errcode)
            } else {
                body_text.clone()
            };

            // 构建详细的错误信息
            let detailed_error = if let Some(errcode) = error_response.errcode {
                let hint = match errcode {
                    5 => "\n💡 Hint: Error code 5 means 'Resource not found'. Possible causes:\n   - The repository path may be incorrect\n   - Pull Requests may not be enabled for this repository\n   - The API endpoint may not be available\n   - Check repository settings at https://cnb.cool",
                    _ => "",
                };
                format!("CNB API error code: {}{}\n📍 Request: {} {}\n📤 Request body: {}\n📥 Response: {}",
                    errcode,
                    hint,
                    "POST",
                    url,
                    request_body.unwrap_or("(empty)"),
                    if !error_response.message.is_empty() { &error_response.message } else { &body_text }
                )
            } else {
                error_msg
            };

            return Err(self.map_error(status, &detailed_error));
        }

        // 使用原始响应体
        let detailed_error = format!(
            "CNB API Error\n📍 Request: {} {}\n📤 Request body: {}\n📥 Response: {}",
            "POST",
            url,
            request_body.unwrap_or("(empty)"),
            body_text
        );
        Err(self.map_error(status, &detailed_error))
    }

    /// 映射 HTTP 状态码到错误类型
    fn map_error(&self, status: u16, message: &str) -> CNBError {
        match status {
            401 => CNBError::AuthenticationFailed,
            403 => {
                if message.contains("rate limit") || message.contains("API rate limit") {
                    CNBError::RateLimitExceeded(message.to_string())
                } else {
                    CNBError::InsufficientPermissions
                }
            }
            404 => CNBError::NotFound(message.to_string()),
            429 => CNBError::RateLimitExceeded(message.to_string()),
            _ => CNBError::ApiError(format!("HTTP {}: {}", status, message)),
        }
    }

    /// 执行 HTTP 请求的通用逻辑
    fn execute<F>(
        &self,
        path: &str,
        body: Option<&Value>,
        request_fn: F,
    ) -> Result<CNBResponse, CNBError>
    where
        F: FnOnce(&HttpClient, &str, RequestConfig) -> Result<HttpResponse, toolkit::HttpError>,
    {
        let url = self.build_url(path)?;
        let client = HttpClient::global()
            .map_err(|e| CNBError::Other(format!("Failed to get HTTP client: {}", e)))?;
        let headers = self.build_headers()?;

        // 调试输出
        toolkit::log_debug!("CNB API Request: URL={}, Headers={:?}", url, headers);

        // 转换 body 为字符串用于调试
        let body_str = body.map(|b| serde_json::to_string(b).unwrap_or_default());
        if let Some(ref body_text) = body_str {
            toolkit::log_debug!("CNB API Request Body: {}", body_text);
        }

        let mut config = RequestConfig::new().headers(headers);
        if let Some(body) = body {
            config = config.body(body);
        }

        let response = request_fn(client, &url, config)
            .map_err(|e| CNBError::Other(format!("Request failed: {}", e)))?;

        self.handle_response(response, &url, body_str.as_deref())
    }
}

impl CNBClient for CNBClientImpl {
    fn get(&self, path: &str) -> Result<CNBResponse, CNBError> {
        self.execute(path, None, |client, url, config| client.get(url, config))
    }

    fn post(&self, path: &str, body: &Value) -> Result<CNBResponse, CNBError> {
        self.execute(path, Some(body), |client, url, config| {
            client.post(url, config)
        })
    }

    fn put(&self, path: &str, body: &Value) -> Result<CNBResponse, CNBError> {
        self.execute(path, Some(body), |client, url, config| {
            client.put(url, config)
        })
    }

    fn patch(&self, path: &str, body: &Value) -> Result<CNBResponse, CNBError> {
        self.execute(path, Some(body), |client, url, config| {
            client.patch(url, config)
        })
    }
}
