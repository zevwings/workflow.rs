use crate::base::http::{HttpClient, HttpResponse, RequestConfig};
use crate::pr::errors::handle_api_error;
use anyhow::{Context, Result};
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

/// Pull Request HTTP 客户端包装器
///
/// 提供统一的 HTTP 请求接口，自动处理错误和响应解析
pub struct PRHttpClient {
    client: &'static HttpClient,
}

impl PRHttpClient {
    /// 创建新的 PR API 客户端
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: HttpClient::global()?,
        })
    }

    /// 执行 GET 请求
    ///
    /// # 参数
    /// * `url` - 请求 URL
    /// * `headers` - 请求头
    ///
    /// # 返回
    /// 解析后的响应数据
    pub fn get<T: DeserializeOwned>(&self, url: &str, headers: &HeaderMap) -> Result<T> {
        let config = RequestConfig::<Value, Value>::new().headers(headers);
        let response = self
            .client
            .get(url, config)
            .context(format!("Failed to send GET request to {}", url))?;

        if !response.is_success() {
            return Err(handle_api_error(&response));
        }

        response
            .as_json()
            .context("Failed to parse response as JSON")
    }

    /// 执行 POST 请求
    ///
    /// # 参数
    /// * `url` - 请求 URL
    /// * `body` - 请求体（可序列化的类型）
    /// * `headers` - 请求头
    ///
    /// # 返回
    /// 解析后的响应数据
    pub fn post<T: DeserializeOwned>(
        &self,
        url: &str,
        body: &impl Serialize,
        headers: &HeaderMap,
    ) -> Result<T> {
        let config = RequestConfig::<_, Value>::new().body(body).headers(headers);
        let response = self
            .client
            .post(url, config)
            .context(format!("Failed to send POST request to {}", url))?;

        if !response.is_success() {
            return Err(handle_api_error(&response));
        }

        response
            .as_json()
            .context("Failed to parse response as JSON")
    }

    /// 执行 PUT 请求
    ///
    /// # 参数
    /// * `url` - 请求 URL
    /// * `body` - 请求体（可序列化的类型）
    /// * `headers` - 请求头
    ///
    /// # 返回
    /// 解析后的响应数据
    pub fn put<T: DeserializeOwned>(
        &self,
        url: &str,
        body: &impl Serialize,
        headers: &HeaderMap,
    ) -> Result<T> {
        let config = RequestConfig::<_, Value>::new().body(body).headers(headers);
        let response = self
            .client
            .put(url, config)
            .context(format!("Failed to send PUT request to {}", url))?;

        if !response.is_success() {
            return Err(handle_api_error(&response));
        }

        response
            .as_json()
            .context("Failed to parse response as JSON")
    }

    /// 执行 PATCH 请求
    ///
    /// # 参数
    /// * `url` - 请求 URL
    /// * `body` - 请求体（可序列化的类型）
    /// * `headers` - 请求头
    ///
    /// # 返回
    /// 解析后的响应数据
    pub fn patch<T: DeserializeOwned>(
        &self,
        url: &str,
        body: &impl Serialize,
        headers: &HeaderMap,
    ) -> Result<T> {
        let config = RequestConfig::<_, Value>::new().body(body).headers(headers);
        let response = self
            .client
            .patch(url, config)
            .context(format!("Failed to send PATCH request to {}", url))?;

        if !response.is_success() {
            return Err(handle_api_error(&response));
        }

        response
            .as_json()
            .context("Failed to parse response as JSON")
    }

    /// 执行 DELETE 请求
    ///
    /// # 参数
    /// * `url` - 请求 URL
    /// * `headers` - 请求头
    ///
    /// # 返回
    /// HTTP 响应（DELETE 请求可能没有响应体）
    pub fn delete(&self, url: &str, headers: &HeaderMap) -> Result<HttpResponse> {
        let config = RequestConfig::<Value, Value>::new().headers(headers);
        let response = self
            .client
            .delete(url, config)
            .context(format!("Failed to send DELETE request to {}", url))?;

        if !response.is_success() {
            return Err(handle_api_error(&response));
        }

        Ok(response)
    }

    /// 执行 GET 请求，返回原始 HttpResponse（用于需要特殊处理的场景）
    ///
    /// # 参数
    /// * `url` - 请求 URL
    /// * `headers` - 请求头
    ///
    /// # 返回
    /// 原始 HTTP 响应
    pub fn get_raw(&self, url: &str, headers: &HeaderMap) -> Result<HttpResponse> {
        let config = RequestConfig::<Value, Value>::new().headers(headers);
        self.client
            .get(url, config)
            .context(format!("Failed to send GET request to {}", url))
    }

    /// 执行 POST 请求，返回原始 HttpResponse（用于需要特殊处理的场景）
    ///
    /// # 参数
    /// * `url` - 请求 URL
    /// * `body` - 请求体（可序列化的类型）
    /// * `headers` - 请求头
    ///
    /// # 返回
    /// 原始 HTTP 响应
    pub fn post_raw(
        &self,
        url: &str,
        body: &impl Serialize,
        headers: &HeaderMap,
    ) -> Result<HttpResponse> {
        let config = RequestConfig::<_, Value>::new().body(body).headers(headers);
        self.client
            .post(url, config)
            .context(format!("Failed to send POST request to {}", url))
    }
}
