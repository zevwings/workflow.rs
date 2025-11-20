//! Jira User REST API
//!
//! 本模块提供了所有用户相关的 REST API 方法。

use anyhow::{Context, Result};

use super::http_client::JiraHttpClient;
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
        let client = JiraHttpClient::global()?;
        client
            .get("myself")
            .context("Failed to get current Jira user")
    }
}
