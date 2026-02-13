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

// use domain::{JiraConfigContext, JiraError};
// use client::{Authorization, HttpClient, HttpMethod};
// use reqwest::Url;
// use serde::Serialize;
// use toolkit::log_debug;

use client::{
    Authorization, HttpClient, HttpClientHolder, JiraClient, JiraClientError, JiraConfigContext,
    JiraRequest, JiraResponse,
};
use reqwest::Url;

use crate::http::RestRequestBuilder;

pub struct JiraClientImpl {
    holder: HttpClientHolder,
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
    fn execute(&self, request: JiraRequest) -> Result<JiraResponse, JiraClientError> {
        // 构建 URL 和认证信息
        let url = self
            .build_url(request.path)
            .map_err(|e| JiraClientError::ApiError(e.to_string()))?;
        let auth = self.build_auth().map_err(|e| JiraClientError::ApiError(e.to_string()))?;

        // 使用 RestRequestBuilder 简化请求构建
        let response = RestRequestBuilder::new(&self.holder, request.method, url)
            .auth(auth)
            .body(request.body)
            .query(request.query)
            .execute()
            .map_err(|e| JiraClientError::ApiError(e.to_string()))?;

        Ok(JiraResponse::new(response))
    }
}

impl JiraClientImpl {
    pub fn new(http_client: Arc<dyn HttpClient>, context: Arc<dyn JiraConfigContext>) -> Self {
        let holder = HttpClientHolder::new(http_client);
        Self { holder, context }
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
    fn build_url(&self, path: String) -> Result<String, JiraClientError> {
        let service_address = self.context.get_jira_service_address();
        if service_address.is_empty() {
            return Err(JiraClientError::ApiError(
                "Jira service address is not configured".to_string(),
            ));
        }

        let base_url = format!("{}/rest/api/2", service_address.trim_end_matches('/'));
        let path_url = format!("{}/{}", base_url, path.trim_start_matches('/'));

        // 使用 Url 构建 URL，自动处理编码
        let url = Url::parse(&path_url).map_err(|e| {
            JiraClientError::ApiError(format!("Failed to parse base URL {}: {}", path_url, e))
        })?;

        Ok(url.to_string())
    }

    /// 构建认证信息
    ///
    /// 从配置提供者获取认证信息，创建 `Authorization` 对象。
    ///
    /// # 返回
    ///
    /// 返回 `Authorization` 对象。
    fn build_auth(&self) -> Result<Authorization, JiraClientError> {
        let email = self.context.get_jira_email();
        let api_token = self.context.get_jira_api_token();

        if email.is_empty() {
            return Err(JiraClientError::ApiError(
                "Jira email is not configured".to_string(),
            ));
        }
        if api_token.is_empty() {
            return Err(JiraClientError::ApiError(
                "Jira API token is not configured".to_string(),
            ));
        }

        Ok(Authorization::basic(email, api_token))
    }
}
