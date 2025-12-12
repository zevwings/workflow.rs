//! Jira Project REST API
//!
//! 本模块提供了所有项目相关的 REST API 方法。

use color_eyre::{
    eyre::{ContextCompat, WrapErr},
    Result,
};
use serde_json::Value;
use std::time::Duration;

use super::helpers::{build_jira_url, jira_auth_config};
use crate::base::http::{HttpClient, RequestConfig};

pub struct JiraProjectApi;

impl JiraProjectApi {
    /// 获取项目的状态列表
    ///
    /// # 参数
    ///
    /// * `project` - Jira 项目名称，如 `"PROJ"`
    ///
    /// # 返回
    ///
    /// 返回状态名称列表。
    ///
    /// # 错误
    ///
    /// 如果项目不存在、无访问权限或 API 调用失败，返回相应的错误信息。
    pub fn get_project_statuses(project: &str) -> Result<Vec<String>> {
        let url = build_jira_url(&format!("project/{}/statuses", project))?;
        let client = HttpClient::global()?;
        let auth = jira_auth_config()?;
        // 设置较短的超时时间（10秒），避免在测试或 CI 环境中长时间等待
        let config =
            RequestConfig::<Value, Value>::new().auth(auth).timeout(Duration::from_secs(10));
        let response = client.get(&url, config)?;
        let data: Value = response
            .ensure_success()?
            .as_json()
            .wrap_err(format!("Failed to fetch project statuses for: {}", project))?;

        let statuses = data
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|version| version.get("statuses"))
            .and_then(|s| s.as_array())
            .wrap_err_with(|| {
                format!(
                    "Invalid statuses JSON structure for project '{}'. The API response format may have changed. Response: {}",
                    project,
                    serde_json::to_string_pretty(&data).unwrap_or_else(|_| "Unable to serialize response".to_string())
                )
            })?;

        let status_names: Vec<String> = statuses
            .iter()
            .filter_map(|s| s.get("name"))
            .filter_map(|n| n.as_str())
            .map(|s| s.to_string())
            .collect();

        Ok(status_names)
    }
}
