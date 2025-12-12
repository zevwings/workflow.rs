//! Jira User REST API
//!
//! 本模块提供了所有用户相关的 REST API 方法。

use color_eyre::{eyre::WrapErr, Result};
use serde_json::Value;

use super::helpers::{build_jira_url, jira_auth_config};
use crate::base::http::{HttpClient, RequestConfig};
use crate::jira::types::JiraUser;

pub struct JiraUserApi;

impl JiraUserApi {
    /// 获取当前用户信息
    ///
    /// 调用 Jira API 的 `/myself` 接口获取当前用户信息。
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
}
