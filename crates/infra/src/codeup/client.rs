use std::{collections::HashMap, sync::Arc};

use client::{
    CodeupClient, CodeupClientError, CodeupConfigContext, CodeupErrorResponse, CodeupRequest,
    CodeupResponse, HttpClientHolder,
};
use client::{HttpClient, HttpResponse};
use serde_json::Value;
use toolkit::log_debug;

use crate::http::RestRequestBuilder;

pub const API_BASE: &str = "https://codeup.aliyun.com";

/// Codeup API 客户端
///
/// 封装 Codeup API 的公共 HTTP 请求逻辑
pub struct CodeupClientImpl {
    holder: HttpClientHolder,
    context: Arc<dyn CodeupConfigContext>,
}

impl CodeupClientImpl {
    pub fn new(http_client: Arc<dyn HttpClient>, context: Arc<dyn CodeupConfigContext>) -> Self {
        let holder = HttpClientHolder::new(http_client);
        Self { holder, context }
    }

    /// 获取 Codeup API 请求的 headers
    fn get_headers(&self) -> Result<HashMap<String, String>, CodeupClientError> {
        let csrf_token = self.context.get_csrf_token()?;
        let cookie = self.context.get_cookie()?;

        let mut headers = HashMap::new();
        headers.insert("X-CSRF-Token".to_string(), csrf_token);
        headers.insert("Cookie".to_string(), cookie);
        headers.insert("Accept".to_string(), "application/json".to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("User-Agent".to_string(), "workflow-cli".to_string());

        Ok(headers)
    }

    /// 将 Response 错误转换为 CodeupClientError
    fn convert_to_codeup_error(&self, response: HttpResponse) -> CodeupClientError {
        // 尝试解析 JSON 错误
        if let Ok(data) = response.json::<Value>() {
            // 尝试解析为 Codeup 错误格式
            if let Ok(error) = serde_json::from_value::<CodeupErrorResponse>(data.clone()) {
                return self.format_from_codeup_error(&error, &response);
            }

            // 如果无法解析为 Codeup 格式，返回 JSON 字符串
            if let Ok(json_str) = serde_json::to_string_pretty(&data) {
                let msg = format!(
                    "Codeup API 请求失败: {}\n\n响应:\n{}",
                    response.status, json_str
                );
                return CodeupClientError::ApiError(msg);
            }
        }

        // 回退到简单错误
        let error_msg = response.get_error_message();
        if let Ok(error_msg) = error_msg {
            return CodeupClientError::ApiError(error_msg);
        }
        CodeupClientError::ApiError(format!("Codeup API 请求失败: {}", response.status))
    }

    /// 格式化 Codeup 错误信息
    fn format_from_codeup_error(
        &self,
        error: &CodeupErrorResponse,
        response: &HttpResponse,
    ) -> CodeupClientError {
        let mut msg = format!(
            "Codeup API 错误: {} (状态码: {})",
            error.message, response.status
        );

        if let Some(error_detail) = &error.error {
            msg.push_str(&format!("\n错误详情: {}", error_detail));
        }

        // 尝试添加完整的错误响应 JSON 以便调试
        if let Ok(data) = response.json::<Value>() {
            if let Ok(json_str) = serde_json::to_string_pretty(&data) {
                msg.push_str(&format!("\n\n完整错误响应:\n{}", json_str));
            }
        } else {
            // 如果无法解析为 JSON，添加提取的错误消息
            let error_msg = response.get_error_message();
            if let Ok(error_msg) = error_msg {
                if !error_msg.is_empty() {
                    msg.push_str(&format!("\n\n错误详情:\n{}", error_msg));
                }
            }
        }

        CodeupClientError::ApiError(msg)
    }

    /// 构建完整的 API URL
    fn build_url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            path.to_string()
        } else {
            format!("{}{}", API_BASE, path)
        }
    }
}

impl CodeupClient for CodeupClientImpl {
    fn execute(&self, request: CodeupRequest) -> Result<CodeupResponse, CodeupClientError> {
        let url = self.build_url(&request.path);
        let headers = self.get_headers()?;

        log_debug!("Codeup API 请求: {} {}", request.method, url);

        // 使用 RestRequestBuilder 简化请求构建
        let response = RestRequestBuilder::new(&self.holder, request.method, url)
            .headers(headers)
            .body(request.body)
            .query(request.query)
            .execute()
            .map_err(|e| CodeupClientError::HttpError(format!("请求失败: {}", e)))?;

        if response.is_success() {
            // 记录成功响应的内容（如果可以解析为 JSON）
            if let Ok(json) = response.json::<Value>() {
                log_debug!("Codeup API 响应体: {}", json);
            }
            Ok(CodeupResponse::new(response))
        } else {
            // 记录错误响应
            log_debug!(
                "Codeup API 错误响应: {}",
                response.get_error_message().map_err(|e| e.to_string()).unwrap_or_default()
            );
            Err(self.convert_to_codeup_error(response))
        }
    }
}
