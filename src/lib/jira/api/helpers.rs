//! Jira API 辅助函数
//!
//! 本模块提供了 Jira API 请求的通用辅助函数，用于减少代码重复。
//! 这些函数使用 OnceLock 缓存配置信息，避免重复读取配置文件。

use crate::base::http::Authorization;
use crate::jira::helpers::{get_auth, get_base_url};
use anyhow::Result;
use std::sync::OnceLock;

/// 获取 Jira API 基础 URL（使用 OnceLock 缓存）
///
/// 从配置文件中读取 Jira 服务地址，并构建 REST API 基础 URL。
/// 格式：`{jira_service_address}/rest/api/2`
///
/// 使用 `OnceLock` 缓存结果，避免重复读取配置文件。
///
/// # 返回
///
/// 返回缓存的 Jira API 基础 URL 字符串。
///
/// # 错误
///
/// 如果 `jira_service_address` 未设置或为空，返回错误。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::jira::api::helpers::jira_base_url;
///
/// let base_url = jira_base_url()?;
/// let url = format!("{}/issue/PROJ-123", base_url);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn jira_base_url() -> Result<&'static str> {
    static BASE_URL: OnceLock<Result<&'static str>> = OnceLock::new();
    BASE_URL
        .get_or_init(|| {
            get_base_url().map(|s| {
                // 将 String 转换为 Box<str>，然后 leak 为 &'static str
                let leaked: &'static str = Box::leak(s.into_boxed_str());
                leaked
            })
        })
        .as_ref()
        .map(|s| *s)
        .map_err(|e| anyhow::anyhow!("Failed to get Jira base URL: {}", e))
}

/// 构建完整的 Jira API URL
///
/// 将 API 路径与基础 URL 组合，构建完整的 Jira API URL。
///
/// # 参数
///
/// * `path` - API 路径（相对于 base_url），如 `"issue/PROJ-123"` 或 `"issue/PROJ-123?fields=*all"`
///
/// # 返回
///
/// 返回完整的 Jira API URL 字符串。
///
/// # 错误
///
/// 如果基础 URL 未配置，返回错误。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::jira::api::helpers::build_jira_url;
///
/// let url = build_jira_url("issue/PROJ-123")?;
/// // 返回: "https://jira.example.com/rest/api/2/issue/PROJ-123"
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn build_jira_url(path: &str) -> Result<String> {
    let base_url = jira_base_url()?;
    Ok(format!("{}/{}", base_url, path))
}

/// 获取 Jira 认证配置（使用 OnceLock 缓存）
///
/// 从配置文件中读取 Jira API 认证所需的 email 和 api_token，
/// 并创建 `Authorization` 对象。
///
/// 使用 `OnceLock` 缓存结果，避免重复读取配置文件。
///
/// # 返回
///
/// 返回缓存的 `Authorization` 对象的静态引用。
///
/// # 错误
///
/// 如果认证信息未配置，返回错误。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::jira::api::helpers::jira_auth_config;
/// use workflow::base::http::RequestConfig;
/// use serde_json::Value;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let auth = jira_auth_config()?;
/// let config = RequestConfig::<Value, Value>::new().auth(auth);
/// # Ok(())
/// # }
/// ```
pub fn jira_auth_config() -> Result<&'static Authorization> {
    static AUTH: OnceLock<Result<Authorization>> = OnceLock::new();
    AUTH.get_or_init(|| {
        let (email, api_token) = get_auth()?;
        Ok(Authorization::new(email, api_token))
    })
    .as_ref()
    .map_err(|e| anyhow::anyhow!("Failed to get Jira auth: {}", e))
}
