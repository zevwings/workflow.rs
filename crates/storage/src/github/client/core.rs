//! GitHub API 客户端
//!
//! 封装 GitHub API 的公共 HTTP 请求逻辑，包括：
//! - 请求头构建
//! - 错误处理
//! - API 基础 URL 管理

use std::{fmt::Write, sync::Arc};

use reqwest::header::HeaderMap;
use serde_json::Value;

use domain::{GitHubContext, GitHubError};
use toolkit::{HttpClient, HttpResponse, RequestConfig};

use crate::github::client::response::{GitHubErrorResponse, GitHubResponse};

pub const API_BASE: &str = "https://api.github.com";

pub trait GitHubClient: Send + Sync {
    /// GET 请求，返回 HTTP 响应
    ///
    /// `path` 应该是相对路径（如 `/repos/owner/repo`），会自动与 API_BASE 拼接
    /// 如果传入完整 URL（以 http:// 或 https:// 开头），则直接使用
    fn get(&self, path: &str) -> Result<GitHubResponse, GitHubError>;

    /// POST 请求，返回 HTTP 响应
    ///
    /// `path` 应该是相对路径（如 `/repos/owner/repo`），会自动与 API_BASE 拼接
    /// 如果传入完整 URL（以 http:// 或 https:// 开头），则直接使用
    fn post(&self, path: &str, body: &Value) -> Result<GitHubResponse, GitHubError>;

    /// PUT 请求，返回 HTTP 响应
    ///
    /// `path` 应该是相对路径（如 `/repos/owner/repo`），会自动与 API_BASE 拼接
    /// 如果传入完整 URL（以 http:// 或 https:// 开头），则直接使用
    fn put(&self, path: &str, body: &Value) -> Result<GitHubResponse, GitHubError>;

    /// PATCH 请求，返回 HTTP 响应
    ///
    /// `path` 应该是相对路径（如 `/repos/owner/repo`），会自动与 API_BASE 拼接
    /// 如果传入完整 URL（以 http:// 或 https:// 开头），则直接使用
    fn patch(&self, path: &str, body: &Value) -> Result<GitHubResponse, GitHubError>;

    /// DELETE 请求，返回 HTTP 响应
    ///
    /// `path` 应该是相对路径（如 `/repos/owner/repo`），会自动与 API_BASE 拼接
    /// 如果传入完整 URL（以 http:// 或 https:// 开头），则直接使用
    fn delete(&self, path: &str) -> Result<GitHubResponse, GitHubError>;
}

/// GitHub API 客户端
///
/// 封装 GitHub API 的公共 HTTP 请求逻辑
pub struct GitHubClientImpl {
    context: Arc<dyn GitHubContext>,
}

impl GitHubClientImpl {
    pub fn new(context: Arc<dyn GitHubContext>) -> Self {
        Self { context }
    }

    /// 获取 GitHub API 请求的 headers
    pub fn get_headers(&self) -> Result<HeaderMap, GitHubError> {
        let token = self.context.get_api_token()?;

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().map_err(|e| {
                GitHubError::ApiError(format!("Failed to parse Authorization header: {}", e))
            })?,
        );
        headers.insert(
            "Accept",
            "application/vnd.github+json".parse().map_err(|e| {
                GitHubError::ApiError(format!("Failed to parse Accept header: {}", e))
            })?,
        );
        headers.insert(
            "X-GitHub-Api-Version",
            "2022-11-28".parse().map_err(|e| {
                GitHubError::ApiError(format!(
                    "Failed to parse X-GitHub-Api-Version header: {}",
                    e
                ))
            })?,
        );
        headers.insert(
            "User-Agent",
            "workflow-cli".parse().map_err(|e| {
                GitHubError::ApiError(format!("Failed to parse User-Agent header: {}", e))
            })?,
        );

        Ok(headers)
    }

    /// 将 HttpResponse 错误转换为 GitHubError
    fn convert_to_github_error(&self, response: HttpResponse) -> GitHubError {
        // 尝试解析 JSON 错误
        if let Ok(data) = response.as_json::<Value>() {
            // 尝试解析为 GitHub 错误格式
            if let Ok(error) = serde_json::from_value::<GitHubErrorResponse>(data.clone()) {
                return self.format_from_github_error(&error, &response);
            }

            // 如果无法解析为 GitHub 格式，返回 JSON 字符串
            if let Ok(json_str) = serde_json::to_string_pretty(&data) {
                let msg = format!(
                    "GitHub API request failed: {} - {}\n\nResponse:\n{}",
                    response.status, response.status_text, json_str
                );
                return GitHubError::ApiError(msg);
            }
        }

        // 回退到简单错误
        let error_msg = response.extract_error_message();
        let msg = if !error_msg.is_empty() {
            format!(
                "GitHub API request failed: {} - {}\n\n{}",
                response.status, response.status_text, error_msg
            )
        } else {
            format!(
                "GitHub API request failed: {} - {}",
                response.status, response.status_text
            )
        };
        GitHubError::ApiError(msg)
    }

    /// 格式化 GitHub 错误信息
    ///
    /// 将 GitHub API 错误响应格式化为用户友好的错误消息
    fn format_from_github_error(
        &self,
        error: &GitHubErrorResponse,
        response: &HttpResponse,
    ) -> GitHubError {
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
        if let Ok(data) = response.as_json::<Value>() {
            if let Ok(json_str) = serde_json::to_string_pretty(&data) {
                writeln!(details, "\nFull error response:\n{}", json_str).ok();
            }
        } else {
            // 如果无法解析为 JSON，添加提取的错误消息
            let error_msg = response.extract_error_message();
            if !error_msg.is_empty() {
                writeln!(details, "\nError details:\n{}", error_msg).ok();
            }
        }

        let mut msg = format!(
            "GitHub API error: {} (Status: {})",
            error.message, response.status
        );
        if !details.is_empty() {
            msg.push_str(&details);
        }
        GitHubError::ApiError(msg)
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

    /// 执行 HTTP 请求的通用逻辑
    fn execute<F>(
        &self,
        path: &str,
        body: Option<&Value>,
        request_fn: F,
    ) -> Result<GitHubResponse, GitHubError>
    where
        F: FnOnce(&HttpClient, &str, RequestConfig) -> Result<HttpResponse, toolkit::HttpError>,
    {
        let url = self.build_url(path);
        let client = HttpClient::global()
            .map_err(|e| GitHubError::ApiError(format!("Failed to get HTTP client: {}", e)))?;
        let headers = self
            .get_headers()
            .map_err(|e| GitHubError::ApiError(format!("Failed to get headers: {}", e)))?;
        let mut config = RequestConfig::new().headers(&headers);
        if let Some(body) = body {
            config = config.body(body);
        }

        let response = request_fn(client, &url, config)
            .map_err(|e| GitHubError::ApiError(format!("Request failed: {}", e)))?;

        if response.is_success() {
            Ok(GitHubResponse::new(response))
        } else {
            Err(self.convert_to_github_error(response))
        }
    }
}

impl GitHubClient for GitHubClientImpl {
    fn get(&self, path: &str) -> Result<GitHubResponse, GitHubError> {
        self.execute(path, None, |client, url, config| client.get(url, config))
    }

    fn post(&self, path: &str, body: &Value) -> Result<GitHubResponse, GitHubError> {
        self.execute(path, Some(body), |client, url, config| {
            client.post(url, config)
        })
    }

    fn put(&self, path: &str, body: &Value) -> Result<GitHubResponse, GitHubError> {
        self.execute(path, Some(body), |client, url, config| {
            client.put(url, config)
        })
    }

    fn patch(&self, path: &str, body: &Value) -> Result<GitHubResponse, GitHubError> {
        self.execute(path, Some(body), |client, url, config| {
            client.patch(url, config)
        })
    }

    fn delete(&self, path: &str) -> Result<GitHubResponse, GitHubError> {
        self.execute(path, None, |client, url, config| client.delete(url, config))
    }
}
