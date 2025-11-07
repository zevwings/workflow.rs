//! HTTP 客户端
//!
//! 本模块提供了 HTTP 客户端的封装，支持多种 HTTP 方法和认证方式。

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

use super::response::HttpResponse;

/// Basic Authentication 认证信息
///
/// 用于 HTTP Basic Authentication 的用户名和密码。
#[derive(Debug, Clone)]
pub struct Authorization {
    /// 用户名（通常是邮箱地址）
    pub username: String,
    /// 密码（通常是 API token）
    pub password: String,
}

impl Authorization {
    /// 创建新的 Authorization
    ///
    /// 创建 Basic Authentication 认证信息。
    ///
    /// # 参数
    ///
    /// * `username` - 用户名（通常是邮箱地址）
    /// * `password` - 密码（通常是 API token）
    ///
    /// # 返回
    ///
    /// 返回 `Authorization` 结构体。
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

/// HTTP 客户端
///
/// 提供 HTTP 请求的封装，支持 GET、POST、PUT、DELETE、PATCH 等方法。
/// 支持 Basic Authentication 和自定义 Headers。
pub struct HttpClient {
    /// 内部的 reqwest 客户端
    client: Client,
}

impl HttpClient {
    /// 创建新的 HttpClient
    ///
    /// 初始化 HTTP 客户端，使用默认配置。
    ///
    /// # 返回
    ///
    /// 返回 `HttpClient` 结构体。
    ///
    /// # 错误
    ///
    /// 如果创建客户端失败，返回相应的错误信息。
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .build()
            .context("Failed to create HTTP client")?;
        Ok(Self { client })
    }

    /// GET 请求
    ///
    /// 发送 HTTP GET 请求到指定 URL。
    ///
    /// # 参数
    ///
    /// * `url` - 请求 URL
    /// * `auth` - 可选的 Basic Authentication 认证信息
    /// * `headers` - 可选的自定义 HTTP Headers
    ///
    /// # 类型参数
    ///
    /// * `T` - 响应数据的类型，必须实现 `Deserialize` trait
    ///
    /// # 返回
    ///
    /// 返回 `HttpResponse<T>` 结构体，包含状态码、响应数据和 Headers。
    ///
    /// # 错误
    ///
    /// 如果请求失败或响应解析失败，返回相应的错误信息。
    pub fn get<T>(
        &self,
        url: &str,
        auth: Option<&Authorization>,
        headers: Option<&HeaderMap>,
    ) -> Result<HttpResponse<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut request = self.client.get(url);

        // 添加 basic auth
        if let Some(auth) = auth {
            request = request.basic_auth(&auth.username, Some(&auth.password));
        }

        // 添加自定义 headers
        if let Some(headers) = headers {
            for (key, value) in headers.iter() {
                request = request.header(key, value);
            }
        }

        let response = request
            .send()
            .with_context(|| format!("Failed to send GET request to: {}", url))?;

        HttpResponse::from_reqwest_response(response)
    }

    /// POST 请求
    ///
    /// 发送 HTTP POST 请求到指定 URL，请求体会被序列化为 JSON。
    ///
    /// # 参数
    ///
    /// * `url` - 请求 URL
    /// * `body` - 请求体，必须实现 `Serialize` trait
    /// * `auth` - 可选的 Basic Authentication 认证信息
    /// * `headers` - 可选的自定义 HTTP Headers
    ///
    /// # 类型参数
    ///
    /// * `T` - 响应数据的类型，必须实现 `Deserialize` trait
    /// * `B` - 请求体的类型，必须实现 `Serialize` trait
    ///
    /// # 返回
    ///
    /// 返回 `HttpResponse<T>` 结构体，包含状态码、响应数据和 Headers。
    ///
    /// # 错误
    ///
    /// 如果请求失败或响应解析失败，返回相应的错误信息。
    pub fn post<T, B>(
        &self,
        url: &str,
        body: &B,
        auth: Option<&Authorization>,
        headers: Option<&HeaderMap>,
    ) -> Result<HttpResponse<T>>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let mut request = self.client.post(url);

        // 添加 basic auth
        if let Some(auth) = auth {
            request = request.basic_auth(&auth.username, Some(&auth.password));
        }

        // 添加自定义 headers
        if let Some(headers) = headers {
            for (key, value) in headers.iter() {
                request = request.header(key, value);
            }
        }

        let response = request
            .json(body)
            .send()
            .with_context(|| format!("Failed to send POST request to: {}", url))?;

        HttpResponse::from_reqwest_response(response)
    }

    /// PUT 请求
    ///
    /// 发送 HTTP PUT 请求到指定 URL，请求体会被序列化为 JSON。
    ///
    /// # 参数
    ///
    /// * `url` - 请求 URL
    /// * `body` - 请求体，必须实现 `Serialize` trait
    /// * `auth` - 可选的 Basic Authentication 认证信息
    /// * `headers` - 可选的自定义 HTTP Headers
    ///
    /// # 类型参数
    ///
    /// * `T` - 响应数据的类型，必须实现 `Deserialize` trait
    /// * `B` - 请求体的类型，必须实现 `Serialize` trait
    ///
    /// # 返回
    ///
    /// 返回 `HttpResponse<T>` 结构体，包含状态码、响应数据和 Headers。
    ///
    /// # 错误
    ///
    /// 如果请求失败或响应解析失败，返回相应的错误信息。
    pub fn put<T, B>(
        &self,
        url: &str,
        body: &B,
        auth: Option<&Authorization>,
        headers: Option<&HeaderMap>,
    ) -> Result<HttpResponse<T>>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let mut request = self.client.put(url);

        // 添加 basic auth
        if let Some(auth) = auth {
            request = request.basic_auth(&auth.username, Some(&auth.password));
        }

        // 添加自定义 headers
        if let Some(headers) = headers {
            for (key, value) in headers.iter() {
                request = request.header(key, value);
            }
        }

        let response = request
            .json(body)
            .send()
            .with_context(|| format!("Failed to send PUT request to: {}", url))?;

        HttpResponse::from_reqwest_response(response)
    }

    /// DELETE 请求
    ///
    /// 发送 HTTP DELETE 请求到指定 URL。
    ///
    /// # 参数
    ///
    /// * `url` - 请求 URL
    /// * `auth` - 可选的 Basic Authentication 认证信息
    /// * `headers` - 可选的自定义 HTTP Headers
    ///
    /// # 类型参数
    ///
    /// * `T` - 响应数据的类型，必须实现 `Deserialize` trait
    ///
    /// # 返回
    ///
    /// 返回 `HttpResponse<T>` 结构体，包含状态码、响应数据和 Headers。
    ///
    /// # 错误
    ///
    /// 如果请求失败或响应解析失败，返回相应的错误信息。
    pub fn delete<T>(
        &self,
        url: &str,
        auth: Option<&Authorization>,
        headers: Option<&HeaderMap>,
    ) -> Result<HttpResponse<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut request = self.client.delete(url);

        // 添加 basic auth
        if let Some(auth) = auth {
            request = request.basic_auth(&auth.username, Some(&auth.password));
        }

        // 添加自定义 headers
        if let Some(headers) = headers {
            for (key, value) in headers.iter() {
                request = request.header(key, value);
            }
        }

        let response = request
            .send()
            .with_context(|| format!("Failed to send DELETE request to: {}", url))?;

        HttpResponse::from_reqwest_response(response)
    }

    /// PATCH 请求
    ///
    /// 发送 HTTP PATCH 请求到指定 URL，请求体会被序列化为 JSON。
    ///
    /// # 参数
    ///
    /// * `url` - 请求 URL
    /// * `body` - 请求体，必须实现 `Serialize` trait
    /// * `auth` - 可选的 Basic Authentication 认证信息
    /// * `headers` - 可选的自定义 HTTP Headers
    ///
    /// # 类型参数
    ///
    /// * `T` - 响应数据的类型，必须实现 `Deserialize` trait
    /// * `B` - 请求体的类型，必须实现 `Serialize` trait
    ///
    /// # 返回
    ///
    /// 返回 `HttpResponse<T>` 结构体，包含状态码、响应数据和 Headers。
    ///
    /// # 错误
    ///
    /// 如果请求失败或响应解析失败，返回相应的错误信息。
    pub fn patch<T, B>(
        &self,
        url: &str,
        body: &B,
        auth: Option<&Authorization>,
        headers: Option<&HeaderMap>,
    ) -> Result<HttpResponse<T>>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let mut request = self.client.patch(url);

        // 添加 basic auth
        if let Some(auth) = auth {
            request = request.basic_auth(&auth.username, Some(&auth.password));
        }

        // 添加自定义 headers
        if let Some(headers) = headers {
            for (key, value) in headers.iter() {
                request = request.header(key, value);
            }
        }

        let response = request
            .json(body)
            .send()
            .with_context(|| format!("Failed to send PATCH request to: {}", url))?;

        HttpResponse::from_reqwest_response(response)
    }

    /// 获取内部 reqwest client（用于自定义请求）
    ///
    /// 返回内部的 `reqwest::blocking::Client`，用于执行自定义的 HTTP 请求。
    ///
    /// # 返回
    ///
    /// 返回 `reqwest::blocking::Client` 的引用。
    pub fn client(&self) -> &Client {
        &self.client
    }
}
