use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

use super::response::HttpResponse;

/// Basic Authentication 认证信息
#[derive(Debug, Clone)]
pub struct Authorization {
    pub username: String,
    pub password: String,
}

impl Authorization {
    /// 创建新的 Authorization
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

/// HTTP 客户端
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    /// 创建新的 HttpClient
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .build()
            .context("Failed to create HTTP client")?;
        Ok(Self { client })
    }

    /// GET 请求
    ///
    /// # 参数
    /// - `url`: 请求 URL
    /// - `auth`: 可选的 basic auth
    /// - `headers`: 可选的自定义 headers
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
    /// # 参数
    /// - `url`: 请求 URL
    /// - `body`: 请求体（会被序列化为 JSON）
    /// - `auth`: 可选的 basic auth
    /// - `headers`: 可选的自定义 headers
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
    /// # 参数
    /// - `url`: 请求 URL
    /// - `body`: 请求体（会被序列化为 JSON）
    /// - `auth`: 可选的 basic auth
    /// - `headers`: 可选的自定义 headers
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
    /// # 参数
    /// - `url`: 请求 URL
    /// - `auth`: 可选的 basic auth
    /// - `headers`: 可选的自定义 headers
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
    /// # 参数
    /// - `url`: 请求 URL
    /// - `body`: 请求体（会被序列化为 JSON）
    /// - `auth`: 可选的 basic auth
    /// - `headers`: 可选的自定义 headers
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
    pub fn client(&self) -> &Client {
        &self.client
    }
}
