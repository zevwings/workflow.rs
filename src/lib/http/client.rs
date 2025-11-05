use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

use super::response::HttpResponse;

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
    pub fn get<T>(&self, url: &str) -> Result<HttpResponse<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self
            .client
            .get(url)
            .send()
            .with_context(|| format!("Failed to send GET request to: {}", url))?;

        HttpResponse::from_reqwest_response(response)
    }

    /// POST 请求
    pub fn post<T, B>(&self, url: &str, body: &B) -> Result<HttpResponse<T>>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let response = self
            .client
            .post(url)
            .json(body)
            .send()
            .with_context(|| format!("Failed to send POST request to: {}", url))?;

        HttpResponse::from_reqwest_response(response)
    }

    /// POST 请求（带自定义 headers）
    pub fn post_with_headers<T, B>(
        &self,
        url: &str,
        body: &B,
        headers: &HeaderMap,
    ) -> Result<HttpResponse<T>>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let mut request = self.client.post(url);

        // 添加自定义 headers
        for (key, value) in headers.iter() {
            request = request.header(key, value);
        }

        let response = request
            .json(body)
            .send()
            .with_context(|| format!("Failed to send POST request to: {}", url))?;

        HttpResponse::from_reqwest_response(response)
    }

    /// PUT 请求
    pub fn put<T, B>(&self, url: &str, body: &B) -> Result<HttpResponse<T>>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let response = self
            .client
            .put(url)
            .json(body)
            .send()
            .with_context(|| format!("Failed to send PUT request to: {}", url))?;

        HttpResponse::from_reqwest_response(response)
    }

    /// DELETE 请求
    pub fn delete<T>(&self, url: &str) -> Result<HttpResponse<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self
            .client
            .delete(url)
            .send()
            .with_context(|| format!("Failed to send DELETE request to: {}", url))?;

        HttpResponse::from_reqwest_response(response)
    }

    /// PATCH 请求
    pub fn patch<T, B>(&self, url: &str, body: &B) -> Result<HttpResponse<T>>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let response = self
            .client
            .patch(url)
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

