//! Jira User REST API
//!
//! 本模块提供了所有用户相关的 REST API 方法。

use color_eyre::{eyre::WrapErr, Result};
use serde_json::Value;

use super::helpers::{build_jira_url, jira_auth_config};
use crate::base::http::{Authorization, HttpClient, RequestConfig};
use crate::jira::types::JiraUser;

pub struct JiraUserApi;

impl JiraUserApi {
    /// 获取当前用户信息
    ///
    /// 调用 Jira API 的 `/myself` 接口获取当前用户信息。
    /// 使用配置文件中的认证信息。
    ///
    /// # 返回
    ///
    /// 返回 `JiraUser` 结构体，包含用户的完整信息。
    pub fn get_current_user() -> Result<JiraUser> {
        let url = build_jira_url("myself")?;
        let client = HttpClient::global()?;
        let auth = jira_auth_config()?;
        let config = RequestConfig::<Value, Value>::new().auth(auth);
        let response = client.get(&url, config)?;
        response.ensure_success()?.as_json().wrap_err("Failed to get current Jira user")
    }

    /// 使用自定义认证信息获取当前用户信息
    ///
    /// 调用 Jira API 的 `/myself` 接口获取当前用户信息。
    /// 使用传入的认证信息，而不是从配置文件读取。
    ///
    /// # 参数
    ///
    /// * `email` - Jira 用户邮箱
    /// * `api_token` - Jira API Token
    /// * `service_address` - Jira 服务地址
    ///
    /// # 返回
    ///
    /// 返回 `JiraUser` 结构体，包含用户的完整信息。
    pub fn get_current_user_with_auth(
        email: &str,
        api_token: &str,
        service_address: &str,
    ) -> Result<JiraUser> {
        let base_url = format!("{}/rest/api/2", service_address);
        let url = format!("{}/myself", base_url);
        let client = HttpClient::global()?;
        let auth = Authorization::new(email, api_token);
        let config = RequestConfig::<Value, Value>::new().auth(&auth);
        let response = client.get(&url, config)?;
        response.ensure_success()?.as_json().wrap_err("Failed to get current Jira user")
    }
}
