use std::{collections::HashMap, fmt::Write, sync::Arc};

use client::{
    GitHubClient, GitHubClientError, GitHubConfigContext, GitHubErrorResponse, GitHubRequest,
    GitHubResponse, HttpClientHolder,
};
use client::{HttpClient, HttpResponse};
use serde_json::Value;
use toolkit::log_debug;

use crate::http::RestRequestBuilder;

pub const API_BASE: &str = "https://api.github.com";

/// GitHub API 客户端
///
/// 封装 GitHub API 的公共 HTTP 请求逻辑
pub struct GitHubClientImpl {
    holder: HttpClientHolder,
    context: Arc<dyn GitHubConfigContext>,
}

impl GitHubClientImpl {
    pub fn new(http_client: Arc<dyn HttpClient>, context: Arc<dyn GitHubConfigContext>) -> Self {
        let holder = HttpClientHolder::new(http_client);
        Self { holder, context }
    }

    /// 获取 GitHub API 请求的 headers
    fn get_headers(&self) -> Result<HashMap<String, String>, GitHubClientError> {
        let token = self.context.get_api_token()?;

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        headers.insert(
            "Accept".to_string(),
            "application/vnd.github+json".to_string(),
        );
        headers.insert("X-GitHub-Api-Version".to_string(), "2022-11-28".to_string());
        headers.insert("User-Agent".to_string(), "workflow-cli".to_string());

        Ok(headers)
    }

    /// 将 Response 错误转换为 GitHubClientError
    fn convert_to_github_error(&self, response: HttpResponse) -> GitHubClientError {
        // 尝试解析 JSON 错误
        if let Ok(data) = response.json::<Value>() {
            // 尝试解析为 GitHub 错误格式
            if let Ok(error) = serde_json::from_value::<GitHubErrorResponse>(data.clone()) {
                return self.format_from_github_error(&error, &response);
            }

            // 如果无法解析为 GitHub 格式，返回 JSON 字符串
            if let Ok(json_str) = serde_json::to_string_pretty(&data) {
                let msg = format!(
                    "GitHub API request failed: {}\n\nResponse:\n{}",
                    response.status, json_str
                );
                return GitHubClientError::ApiError(msg);
            }
        }

        // 回退到简单错误
        let error_msg = response.get_error_message();
        if let Ok(error_msg) = error_msg {
            return GitHubClientError::ApiError(error_msg);
        }
        GitHubClientError::ApiError(format!("GitHub API request failed: {}", response.status))
    }

    /// 格式化 GitHub 错误信息
    ///
    /// 将 GitHub API 错误响应格式化为用户友好的错误消息
    fn format_from_github_error(
        &self,
        error: &GitHubErrorResponse,
        response: &HttpResponse,
    ) -> GitHubClientError {
        let mut details = String::new();

        if let Some(errors) = &error.errors {
            for err in errors {
                if let (Some(resource), Some(field), Some(code)) =
                    (&err.resource, &err.field, &err.code)
                {
                    writeln!(
                        details,
                        "  - {}: {} field is invalid ({})",
                        resource, field, code
                    )
                    .ok();
                }
            }
        }

        // 尝试添加完整的错误响应 JSON 以便调试
        if let Ok(data) = response.json::<Value>() {
            if let Ok(json_str) = serde_json::to_string_pretty(&data) {
                writeln!(details, "\nFull error response:\n{}", json_str).ok();
            }
        } else {
            // 如果无法解析为 JSON，添加提取的错误消息
            let error_msg = response.get_error_message();
            if let Ok(error_msg) = error_msg {
                if !error_msg.is_empty() {
                    writeln!(details, "\nError details:\n{}", error_msg).ok();
                }
            }
        }

        let mut msg = format!(
            "GitHub API error: {} (Status: {})",
            error.message, response.status
        );
        if !details.is_empty() {
            msg.push_str(&details);
        }
        GitHubClientError::ApiError(msg)
    }

    /// 构建完整的 API URL
    ///
    /// 如果传入的路径已经是完整 URL（以 http:// 或 https:// 开头），则直接返回
    /// 否则将相对路径与 API_BASE 拼接
    fn build_url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            path.to_string()
        } else {
            format!("{}{}", API_BASE, path)
        }
    }
}

impl GitHubClient for GitHubClientImpl {
    fn execute(&self, request: GitHubRequest) -> Result<GitHubResponse, GitHubClientError> {
        let url = self.build_url(&request.path);
        let headers = self.get_headers()?;

        // 使用 RestRequestBuilder 简化请求构建
        let response = RestRequestBuilder::new(&self.holder, request.method, url)
            .headers(headers)
            .body(request.body)
            .query(request.query)
            .execute()
            .map_err(|e| GitHubClientError::ApiError(format!("Request failed: {}", e)))?;

        if response.is_success() {
            // 记录成功响应的内容（如果可以解析为 JSON）
            if let Ok(json) = response.json::<Value>() {
                log_debug!("GitHub API response body: {}", json);
            }
            Ok(GitHubResponse::new(response))
        } else {
            // 记录错误响应
            log_debug!(
                "GitHub API error response: {}",
                response.get_error_message().map_err(|e| e.to_string()).unwrap_or_default()
            );
            Err(self.convert_to_github_error(response))
        }
    }
}
