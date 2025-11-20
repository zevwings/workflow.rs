//! Jira HTTP 客户端
//!
//! 本模块提供了统一的 Jira REST API 请求接口，使用单例模式缓存认证信息和客户端实例。
//! 所有 Jira API 调用都应该通过此客户端进行。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;

use crate::base::http::{Authorization, HttpClient, RequestConfig};
use crate::jira::helpers::{get_auth, get_base_url};

/// Jira HTTP 客户端
///
/// 提供统一的 Jira REST API 请求接口，使用单例模式缓存认证信息和客户端实例。
pub struct JiraHttpClient {
    email: String,
    api_token: String,
    base_url: String,
}

impl JiraHttpClient {
    /// 获取全局 JiraHttpClient 单例
    ///
    /// 返回进程级别的 JiraHttpClient 单例。
    /// 单例会在首次调用时初始化，后续调用会复用同一个实例。
    ///
    /// # 返回
    ///
    /// 返回 `JiraHttpClient` 的静态引用。
    ///
    /// # 错误
    ///
    /// 如果认证信息或基础 URL 未配置，返回错误。
    pub fn global() -> Result<&'static Self> {
        static CLIENT: OnceLock<Result<JiraHttpClient>> = OnceLock::new();
        CLIENT.get_or_init(|| {
            let (email, api_token) = match get_auth() {
                Ok(auth) => auth,
                Err(e) => return Err(anyhow::anyhow!("Failed to get Jira authentication: {}", e)),
            };
            let base_url = match get_base_url() {
                Ok(url) => url,
                Err(e) => return Err(anyhow::anyhow!("Failed to get Jira base URL: {}", e)),
            };
            HttpClient::global()
                .map_err(|e| anyhow::anyhow!("Failed to get HTTP client: {}", e))?;

            Ok(JiraHttpClient {
                email,
                api_token,
                base_url,
            })
        })
        .as_ref()
        .map_err(|e| anyhow::anyhow!("Failed to initialize JiraHttpClient: {}", e))
    }

    /// 执行 GET 请求
    ///
    /// # 参数
    ///
    /// * `path` - API 路径（相对于 base_url），如 `"issue/PROJ-123"`
    ///
    /// # 返回
    ///
    /// 返回反序列化后的响应数据。
    pub fn get<T>(&self, path: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.base_url, path);
        let auth = Authorization::new(&self.email, &self.api_token);
        let config = RequestConfig::<Value, Value>::new().auth(&auth);
        let http_client = HttpClient::global()?;
        let response = http_client
            .get(&url, config)
            .context(format!("Failed to GET {}", path))?;

        let response = response.ensure_success()?;
        response.as_json()
    }

    /// 执行 POST 请求
    ///
    /// # 参数
    ///
    /// * `path` - API 路径（相对于 base_url）
    /// * `body` - 请求体（会被序列化为 JSON）
    ///
    /// # 返回
    ///
    /// 返回反序列化后的响应数据。
    pub fn post<Req, Resp>(&self, path: &str, body: &Req) -> Result<Resp>
    where
        Req: Serialize,
        Resp: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.base_url, path);
        let auth = Authorization::new(&self.email, &self.api_token);
        let config = RequestConfig::<Req, Value>::new()
            .body(body)
            .auth(&auth);
        let http_client = HttpClient::global()?;
        let response = http_client
            .post(&url, config)
            .context(format!("Failed to POST {}", path))?;

        let response = response.ensure_success()?;
        response.as_json()
    }

    /// 执行 PUT 请求
    ///
    /// # 参数
    ///
    /// * `path` - API 路径（相对于 base_url）
    /// * `body` - 请求体（会被序列化为 JSON）
    ///
    /// # 返回
    ///
    /// 返回反序列化后的响应数据。
    pub fn put<Req, Resp>(&self, path: &str, body: &Req) -> Result<Resp>
    where
        Req: Serialize,
        Resp: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.base_url, path);
        let auth = Authorization::new(&self.email, &self.api_token);
        let config = RequestConfig::<Req, Value>::new()
            .body(body)
            .auth(&auth);
        let http_client = HttpClient::global()?;
        let response = http_client
            .put(&url, config)
            .context(format!("Failed to PUT {}", path))?;

        let response = response.ensure_success()?;
        response.as_json()
    }
}

