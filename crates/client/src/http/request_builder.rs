//! HTTP 请求 Builder（定义层）
//!
//! 链式 API：get/post、.body()、.auth()、.multipart()、.send()。
//! 由 client 定义，外部通过 `dyn HttpClient` 使用，不依赖 infra。

use std::{collections::HashMap, time::Duration};

use super::{Authorization, ErrorContext, HttpClient, HttpError, HttpMethod, MultipartRequest};
use super::{HttpRequest, HttpResponse};

/// HTTP 请求 Builder
///
/// 通过 `client.get()` / `client.post()` 等创建，链式设置参数后 `send()` 执行。
pub struct RequestBuilder<'a> {
    client: &'a dyn HttpClient,
    method: HttpMethod,
    url: String,
    body: Option<serde_json::Value>,
    query: Option<serde_json::Value>,
    headers: HashMap<String, String>,
    auth: Option<Authorization>,
    timeout: Option<Duration>,
    multipart: Option<MultipartRequest>,
}

impl<'a> RequestBuilder<'a> {
    /// 创建 Builder（由 trait 方法调用）
    pub fn new(client: &'a dyn HttpClient, method: HttpMethod, url: impl Into<String>) -> Self {
        Self {
            client,
            method,
            url: url.into(),
            body: None,
            query: None,
            headers: HashMap::new(),
            auth: None,
            timeout: None,
            multipart: None,
        }
    }

    /// 设置请求体（JSON）
    pub fn body<T: serde::Serialize>(mut self, body: &T) -> Result<Self, HttpError> {
        self.body = Some(
            serde_json::to_value(body).map_err(|e| HttpError::RequestBuild {
                message: e.to_string(),
                context: ErrorContext::new(&self.url, self.method).into_box(),
            })?,
        );
        Ok(self)
    }

    /// 设置查询参数
    pub fn query<T: serde::Serialize>(mut self, query: &T) -> Result<Self, HttpError> {
        self.query = Some(
            serde_json::to_value(query).map_err(|e| HttpError::RequestBuild {
                message: e.to_string(),
                context: ErrorContext::new(&self.url, self.method).into_box(),
            })?,
        );
        Ok(self)
    }

    /// 添加请求头
    pub fn header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.headers.insert(name.as_ref().to_lowercase(), value.as_ref().to_string());
        self
    }

    /// 设置认证
    pub fn auth(mut self, auth: Authorization) -> Self {
        self.auth = Some(auth);
        self
    }

    /// 设置超时
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// 设置 Multipart 表单
    ///
    /// 使用 multipart 时会忽略 body 设置。
    pub fn multipart(mut self, request: MultipartRequest) -> Self {
        self.multipart = Some(request);
        self
    }

    /// 发送请求
    pub fn send(self) -> Result<HttpResponse, HttpError> {
        if let Some(multipart) = self.multipart {
            return self.client.execute_multipart(
                self.method,
                &self.url,
                multipart,
                self.query,
                self.headers,
                self.auth,
                self.timeout,
            );
        }

        let req = HttpRequest {
            method: self.method,
            url: self.url,
            headers: self.headers,
            query: self.query,
            body: self.body,
            auth: self.auth,
            timeout: self.timeout,
        };
        self.client.execute(req)
    }

    /// 发送请求并解析为 JSON
    pub fn json<T: serde::de::DeserializeOwned>(self) -> Result<T, HttpError> {
        let response = self.send()?;
        serde_json::from_slice(response.bytes()).map_err(|e| HttpError::ResponseParse {
            message: format!("Failed to parse JSON: {}", e),
            context: ErrorContext::new(&response.url, response.method)
                .with_response_status(response.status)
                .with_response_headers(response.headers.clone())
                .into_box(),
        })
    }

    /// 发送请求并获取文本
    pub fn text(self) -> Result<String, HttpError> {
        let response = self.send()?;
        String::from_utf8(response.body).map_err(|e| HttpError::ResponseParse {
            message: e.to_string(),
            context: ErrorContext::new(&response.url, response.method)
                .with_response_status(response.status)
                .with_response_headers(response.headers.clone())
                .into_box(),
        })
    }
}
