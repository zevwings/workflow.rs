//! Jira 客户端
//!
//! 本模块提供了 Jira REST API 客户端实现，用于统一发送和解析 Jira 的数据。
//!
//! ## 配置提供者
//!
//! Jira 客户端通过 `JiraConfigProvider` trait 获取配置，实现了依赖倒置原则。
//! 默认情况下，`JiraClient::global()` 使用 `SettingsAdapter` 从配置文件读取设置。
//! 也可以通过 `JiraClient::with_config()` 使用自定义配置提供者（主要用于测试）。

use std::sync::Arc;

use serde::Serialize;

use domain::{jira::JiraConfigContext, JiraError};
use toolkit::{Authorization, HttpClient, HttpMethod, RequestConfig};

use crate::jira::client::types::JiraResponse;

pub trait JiraClient: Send + Sync {
    /// 发送 GET 请求到 Jira API
    ///
    /// # 参数
    ///
    /// * `path` - API 路径（相对于 base_url），如 `"issue/PROJ-123"` 或 `"myself"`
    /// * `query` - 可选的查询参数
    fn get(
        &self,
        path: &str,
        query: Option<&[(String, String)]>,
    ) -> Result<JiraResponse, JiraError>;

    /// 发送 POST 请求到 Jira API
    ///
    /// # 参数
    ///
    /// * `path` - API 路径（相对于 base_url），如 `"issue/PROJ-123/transitions"`
    /// * `body` - 请求体（JSON 值）
    /// * `query` - 可选的查询参数
    fn post(
        &self,
        path: &str,
        body: &serde_json::Value,
        query: Option<&[(String, String)]>,
    ) -> Result<JiraResponse, JiraError>;
}

pub struct JiraClientImpl {
    context: Arc<dyn JiraConfigContext>,
}

/// Jira 客户端
///
/// 所有 Jira API 调用使用同一个客户端实现，通过配置提供者区分不同的配置源。
/// 所有配置（URL、认证信息）都从配置提供者动态获取。
///
/// 支持两种使用方式：
/// 1. 使用全局单例（`global()`）：自动使用默认配置适配器
/// 2. 使用自定义配置提供者（`with_config()`）：支持测试和自定义配置源
impl JiraClient for JiraClientImpl {
    fn get(
        &self,
        path: &str,
        query: Option<&[(String, String)]>,
    ) -> Result<JiraResponse, JiraError> {
        self.request(HttpMethod::Get, path, None::<&()>, query)
    }

    fn post(
        &self,
        path: &str,
        body: &serde_json::Value,
        query: Option<&[(String, String)]>,
    ) -> Result<JiraResponse, JiraError> {
        self.request(HttpMethod::Post, path, Some(body), query)
    }
}

impl JiraClientImpl {
    pub fn new(context: Arc<dyn JiraConfigContext>) -> Self {
        Self { context }
    }

    /// 执行 HTTP 请求的核心逻辑
    ///
    /// 所有 GET 和 POST 请求共享的实现逻辑。
    ///
    /// # 参数
    ///
    /// * `method` - HTTP 方法枚举（`HttpMethod::Get` 或 `HttpMethod::Post`）
    /// * `path` - API 路径
    /// * `body` - 可选的请求体（JSON 值）
    /// * `query` - 可选的查询参数
    fn request<T: Serialize>(
        &self,
        method: HttpMethod,
        path: &str,
        body: Option<&T>,
        query: Option<&[(String, String)]>,
    ) -> Result<JiraResponse, JiraError> {
        // 构建 URL
        let url = self.build_url(path, query).map_err(|e| JiraError::ApiError(e.to_string()))?;

        // 构建认证信息
        let auth = self.build_auth().map_err(|e| JiraError::ApiError(e.to_string()))?;

        // 获取 HTTP 客户端
        let client = HttpClient::global().map_err(|e| JiraError::ApiError(e.to_string()))?;

        // 构建请求配置
        let mut request_config = RequestConfig::new().auth(auth);

        // 添加请求体（如果有）
        if let Some(body) = body {
            let body_value = serde_json::to_value(body).map_err(|e| {
                JiraError::ApiError(format!("Failed to serialize request body: {}", e))
            })?;
            request_config = request_config.body(&body_value);
            toolkit::log_debug!("Jira request: {} {}", method, url);
            toolkit::log_debug!("Jira request body: {}", body_value);
        } else {
            toolkit::log_debug!("Jira request: {} {}", method, url);
        }

        // 发送请求
        let response = match method {
            HttpMethod::Get => client.get(&url, request_config),
            HttpMethod::Post => client.post(&url, request_config),
            _ => {
                return Err(JiraError::ApiError(format!(
                    "Unsupported HTTP method: {}. Only GET and POST are supported.",
                    method
                )))
            }
        }
        .map_err(|e| JiraError::ApiError(format!("{} {}: {}", method, url, e)))?;

        // 检查错误（使用 ensure_success_with 统一处理）
        let response = response
            .ensure_success_with(|r| {
                let error_message = r.extract_error_message();
                format!("Jira API request failed: {} - {}", r.status, error_message)
            })
            .map_err(|e| JiraError::ApiError(e.to_string()))?;

        // 解析 JSON 响应
        let data = response
            .as_json::<serde_json::Value>()
            .map_err(|e| JiraError::ApiError(e.to_string()))?;
        Ok(JiraResponse::new(data))
    }

    /// 构建 Jira API URL
    ///
    /// 从配置提供者获取服务地址，并构建完整的 API URL。
    /// 格式：`{jira_service_address}/rest/api/2/{path}`
    ///
    /// # 参数
    ///
    /// * `path` - API 路径（相对于 base_url）
    /// * `query` - 可选的查询参数
    ///
    /// # 返回
    ///
    /// 返回完整的 Jira API URL。
    fn build_url(
        &self,
        path: &str,
        query: Option<&[(String, String)]>,
    ) -> Result<String, JiraError> {
        let service_address = self.context.get_jira_service_address();
        if service_address.is_empty() {
            return Err(JiraError::ApiError(
                "Jira service address is not configured".to_string(),
            ));
        }

        let base_url = format!("{}/rest/api/2", service_address.trim_end_matches('/'));
        let path_url = format!("{}/{}", base_url, path.trim_start_matches('/'));

        // 使用 reqwest::Url 来构建 URL，自动处理编码
        let mut url = reqwest::Url::parse(&path_url).map_err(|e| {
            JiraError::ApiError(format!("Failed to parse base URL {}: {}", path_url, e))
        })?;

        // 添加查询参数
        if let Some(query_params) = query {
            if !query_params.is_empty() {
                let mut query_pairs = url.query_pairs_mut();
                for (k, v) in query_params {
                    query_pairs.append_pair(k, v);
                }
            }
        }

        Ok(url.to_string())
    }

    /// 构建认证信息
    ///
    /// 从配置提供者获取认证信息，创建 `Authorization` 对象。
    ///
    /// # 返回
    ///
    /// 返回 `Authorization` 对象。
    fn build_auth(&self) -> Result<Authorization, JiraError> {
        let email = self.context.get_jira_email();
        let api_token = self.context.get_jira_api_token();

        if email.is_empty() {
            return Err(JiraError::ApiError(
                "Jira email is not configured".to_string(),
            ));
        }
        if api_token.is_empty() {
            return Err(JiraError::ApiError(
                "Jira API token is not configured".to_string(),
            ));
        }

        Ok(Authorization::basic(email, api_token))
    }
}
