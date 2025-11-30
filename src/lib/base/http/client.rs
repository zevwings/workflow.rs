//! HTTP 客户端
//!
//! 本模块提供了 HTTP 客户端的封装，支持多种 HTTP 方法和认证方式。
//! 支持 Basic Authentication 和自定义 Headers。

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::Value;
use std::sync::OnceLock;
use std::time::Duration;

use super::config::RequestConfig;
use super::method::HttpMethod;
use super::response::HttpResponse;

/// HTTP 客户端
///
/// 提供 HTTP 请求的封装，支持 GET、POST、PUT、DELETE、PATCH 等方法。
/// 支持 Basic Authentication 和自定义 Headers。
pub struct HttpClient {
    /// 内部的 reqwest 客户端
    client: Client,
}

impl HttpClient {
    /// 创建新的 HttpClient（私有方法）
    ///
    /// 初始化 HTTP 客户端，使用默认配置。
    /// 支持从环境变量读取代理配置（http_proxy, https_proxy, all_proxy）。
    /// 此方法仅在 `global()` 方法内部使用，用于初始化全局单例。
    ///
    /// # 返回
    ///
    /// 返回 `HttpClient` 结构体。
    ///
    /// # 错误
    ///
    /// 如果创建客户端失败，返回相应的错误信息。
    fn new() -> Result<Self> {
        let builder = Client::builder();
        let client = builder.build().context("Failed to create HTTP client")?;
        Ok(Self { client })
    }

    /// 获取全局 HttpClient 单例
    ///
    /// 返回进程级别的 HttpClient 单例，使用默认配置。
    /// 单例会在首次调用时初始化，后续调用会复用同一个实例。
    ///
    /// # 返回
    ///
    /// 返回 `HttpClient` 的静态引用。
    ///
    /// # 错误
    ///
    /// 如果创建客户端失败，返回相应的错误信息。
    ///
    /// # 优势
    ///
    /// - 复用连接池：所有请求共享同一个连接池，提高性能
    /// - 减少资源消耗：避免重复创建客户端实例
    /// - 线程安全：可以在多线程环境中安全使用
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use crate::base::http::HttpClient;
    ///
    /// let client = HttpClient::global()?;
    /// let response = client.get("https://api.example.com", None, None)?;
    /// ```
    pub fn global() -> Result<&'static Self> {
        static CLIENT: OnceLock<Result<HttpClient>> = OnceLock::new();
        CLIENT
            .get_or_init(HttpClient::new)
            .as_ref()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))
    }

    /// 构建 HTTP 请求（内部辅助方法）
    ///
    /// 根据指定的 HTTP 方法、URL 和配置构建请求。
    ///
    /// # 参数
    ///
    /// * `method` - HTTP 方法
    /// * `url` - 请求 URL
    /// * `config` - 请求配置，包含可选的请求体、查询参数、认证信息、Headers 和超时时间
    ///
    /// # 类型参数
    ///
    /// * `B` - 请求体的类型，必须实现 `Serialize` trait
    /// * `Q` - 查询参数的类型，必须实现 `Serialize` trait
    ///
    /// # 返回
    ///
    /// 返回配置好的 `RequestBuilder`。
    fn build_request<B, Q>(
        &self,
        method: HttpMethod,
        url: &str,
        config: RequestConfig<B, Q>,
    ) -> reqwest::blocking::RequestBuilder
    where
        B: Serialize,
        Q: Serialize + ?Sized,
    {
        let mut request = match method {
            HttpMethod::Get => self.client.get(url),
            HttpMethod::Post => self.client.post(url),
            HttpMethod::Put => self.client.put(url),
            HttpMethod::Delete => self.client.delete(url),
            HttpMethod::Patch => self.client.patch(url),
        };

        // 添加 body（如果有）
        if let Some(body) = config.body {
            request = request.json(body);
        }

        // 添加 query 参数
        if let Some(query) = config.query {
            request = request.query(query);
        }

        // 添加 auth
        if let Some(auth) = config.auth {
            request = request.basic_auth(&auth.username, Some(&auth.password));
        }

        // 添加 headers
        if let Some(headers) = config.headers {
            for (key, value) in headers.iter() {
                request = request.header(key, value);
            }
        }

        // 设置超时（如果提供了则使用，否则使用默认 30 秒）
        let timeout_duration = config.timeout.unwrap_or_else(|| Duration::from_secs(30));
        request = request.timeout(timeout_duration);

        request
    }

    /// GET 请求
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use crate::base::http::{HttpClient, RequestConfig};
    ///
    /// let client = HttpClient::global()?;
    /// let config = RequestConfig::<Value, _>::new()
    ///     .query(&[("page", "1")]);
    /// let response = client.get("https://api.example.com", config)?;
    /// ```
    pub fn get<Q>(&self, url: &str, config: RequestConfig<Value, Q>) -> Result<HttpResponse>
    where
        Q: Serialize + ?Sized,
    {
        let response = self
            .build_request(HttpMethod::Get, url, config)
            .send()
            .with_context(|| format!("Failed to send GET request to: {}", url))?;

        HttpResponse::from_reqwest_response(response)
    }

    /// POST 请求
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use crate::base::http::{HttpClient, RequestConfig};
    ///
    /// let client = HttpClient::global()?;
    /// let body = serde_json::json!({"key": "value"});
    /// let config = RequestConfig::new().body(&body);
    /// let response = client.post("https://api.example.com", config)?;
    /// ```
    pub fn post<B, Q>(&self, url: &str, config: RequestConfig<B, Q>) -> Result<HttpResponse>
    where
        B: Serialize,
        Q: Serialize + ?Sized,
    {
        let response = self
            .build_request(HttpMethod::Post, url, config)
            .send()
            .with_context(|| format!("Failed to send POST request to: {}", url))?;

        HttpResponse::from_reqwest_response(response)
    }

    /// PUT 请求
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use crate::base::http::{HttpClient, RequestConfig};
    ///
    /// let client = HttpClient::global()?;
    /// let body = serde_json::json!({"key": "value"});
    /// let config = RequestConfig::new().body(&body);
    /// let response = client.put("https://api.example.com", config)?;
    /// ```
    pub fn put<B, Q>(&self, url: &str, config: RequestConfig<B, Q>) -> Result<HttpResponse>
    where
        B: Serialize,
        Q: Serialize + ?Sized,
    {
        let response = self
            .build_request(HttpMethod::Put, url, config)
            .send()
            .with_context(|| format!("Failed to send PUT request to: {}", url))?;

        HttpResponse::from_reqwest_response(response)
    }

    /// DELETE 请求
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use crate::base::http::{HttpClient, RequestConfig};
    ///
    /// let client = HttpClient::global()?;
    /// let config = RequestConfig::<Value, _>::new();
    /// let response = client.delete("https://api.example.com", config)?;
    /// ```
    pub fn delete<Q>(&self, url: &str, config: RequestConfig<Value, Q>) -> Result<HttpResponse>
    where
        Q: Serialize + ?Sized,
    {
        let response = self
            .build_request(HttpMethod::Delete, url, config)
            .send()
            .with_context(|| format!("Failed to send DELETE request to: {}", url))?;

        HttpResponse::from_reqwest_response(response)
    }

    /// PATCH 请求
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use crate::base::http::{HttpClient, RequestConfig};
    ///
    /// let client = HttpClient::global()?;
    /// let body = serde_json::json!({"key": "value"});
    /// let config = RequestConfig::new().body(&body);
    /// let response = client.patch("https://api.example.com", config)?;
    /// ```
    pub fn patch<B, Q>(&self, url: &str, config: RequestConfig<B, Q>) -> Result<HttpResponse>
    where
        B: Serialize,
        Q: Serialize + ?Sized,
    {
        let response = self
            .build_request(HttpMethod::Patch, url, config)
            .send()
            .with_context(|| format!("Failed to send PATCH request to: {}", url))?;

        HttpResponse::from_reqwest_response(response)
    }

    /// 流式请求
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use crate::base::http::{HttpClient, HttpMethod, RequestConfig};
    /// use std::io::Read;
    ///
    /// let client = HttpClient::global()?;
    /// let config = RequestConfig::<Value, _>::new()
    ///     .query(&[("page", "1")]);
    /// let mut response = client.stream(HttpMethod::Get, "https://example.com/api", config)?;
    /// let mut buffer = vec![0u8; 8192];
    /// response.read(&mut buffer)?;
    /// ```
    pub fn stream<B, Q>(
        &self,
        method: HttpMethod,
        url: &str,
        config: RequestConfig<B, Q>,
    ) -> Result<reqwest::blocking::Response>
    where
        B: Serialize,
        Q: Serialize + ?Sized,
    {
        self.build_request(method, url, config)
            .send()
            .with_context(|| format!("Failed to send {} request to: {}", method, url))
    }
}
