//! HTTP 请求

use std::time::{Duration, Instant};

use reqwest::blocking::multipart::Form;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::auth::Authorization;
use super::client::HttpClient;
use super::error::{ErrorContext, HttpError};
use super::headers::IntoHeaderMap;
use super::method::HttpMethod;
use super::multipart::MultipartRequest;
use super::response::Response;
use super::retry::{RetryConfig, RetryExecutor};

/// HTTP 请求
///
/// 提供链式 API 构建和发送 HTTP 请求。
pub struct Request<'a> {
    client: &'a HttpClient,
    method: HttpMethod,
    url: String,
    // 请求配置
    body: Option<serde_json::Value>,
    query: Option<serde_json::Value>,
    headers: Option<HeaderMap>,
    auth: Option<Authorization>,
    timeout: Option<Duration>,
    retry: Option<RetryConfig>,
    multipart: Option<Form>,
}

impl<'a> Request<'a> {
    /// 创建请求（内部使用）
    pub(crate) fn new(client: &'a HttpClient, method: HttpMethod, url: impl Into<String>) -> Self {
        Self {
            client,
            method,
            url: url.into(),
            body: None,
            query: None,
            headers: None,
            auth: None,
            timeout: None,
            retry: None,
            multipart: None,
        }
    }

    /// 设置请求体（JSON）
    pub fn body<T: Serialize>(mut self, body: &T) -> Self {
        self.body = serde_json::to_value(body).ok();
        self
    }

    /// 设置查询参数
    pub fn query<T: Serialize>(mut self, query: &T) -> Self {
        self.query = serde_json::to_value(query).ok();
        self
    }

    /// 设置请求头
    pub fn headers<H: IntoHeaderMap>(mut self, headers: H) -> Self {
        match headers.into_header_map() {
            Ok(map) => {
                self.headers = Some(map);
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to convert headers, ignoring");
            }
        }
        self
    }

    /// 添加单个请求头
    pub fn header(mut self, name: &str, value: &str) -> Self {
        let headers = self.headers.get_or_insert_with(HeaderMap::new);
        if let (Ok(name), Ok(value)) = (HeaderName::try_from(name), HeaderValue::from_str(value)) {
            headers.insert(name, value);
        }
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

    /// 设置重试配置
    ///
    /// 注意：Multipart 请求不支持重试（Form 不可 Clone）。
    pub fn retry(mut self, config: RetryConfig) -> Self {
        self.retry = Some(config);
        self
    }

    /// 设置 Multipart 表单
    ///
    /// 使用 multipart 时，会忽略 body 设置。
    /// 注意：Multipart 请求不支持重试（Form 不可 Clone）。
    pub fn multipart(mut self, mut request: MultipartRequest) -> Self {
        // 先提取其他设置，再消费 form
        if request.query.is_some() {
            self.query = request.query.take();
        }
        if request.headers.is_some() {
            self.headers = request.headers.take();
        }
        if request.auth.is_some() {
            self.auth = request.auth.take();
        }
        if request.timeout.is_some() {
            self.timeout = request.timeout.take();
        }
        // 最后消费 form
        self.multipart = request.into_form();
        self
    }

    /// 发送请求并获取响应
    pub fn send(mut self) -> Result<Response, HttpError> {
        // Multipart 请求不支持重试（Form 不可 Clone）
        if self.multipart.is_some() {
            if self.retry.is_some() {
                tracing::warn!("Retry is not supported for multipart requests, ignoring");
            }
            return self.execute_once();
        }

        if let Some(retry_config) = self.retry.take() {
            let executor = RetryExecutor::new(&retry_config);
            executor.execute(|| self.execute_for_retry())
        } else {
            self.execute_once()
        }
    }

    /// 发送请求并直接解析为 JSON
    ///
    /// 这是 `send()?.json()` 的快捷方式。
    pub fn json<T: DeserializeOwned>(self) -> Result<T, HttpError> {
        self.send()?.json()
    }

    /// 发送请求并直接获取文本
    ///
    /// 这是 `send()?.into_text()` 的快捷方式。
    pub fn text(self) -> Result<String, HttpError> {
        self.send()?.into_text()
    }

    /// 发送请求并获取原始响应（流式）
    pub fn stream(mut self) -> Result<reqwest::blocking::Response, HttpError> {
        let start = Instant::now();
        let request = self.build_request_mut()?;

        let response = request.send().map_err(|e| {
            let context = self.create_error_context(start.elapsed());
            self.convert_reqwest_error(e, context)
        })?;

        Ok(response)
    }

    /// 执行单次请求（消费 self）
    fn execute_once(mut self) -> Result<Response, HttpError> {
        let start = Instant::now();
        let request = self.build_request_mut()?;

        let response = request.send().map_err(|e| {
            let context = self.create_error_context(start.elapsed());
            self.convert_reqwest_error(e, context)
        })?;

        let duration = start.elapsed();

        Response::from_reqwest(
            response,
            self.method,
            duration,
            self.client.max_response_body_size,
        )
    }

    /// 执行请求（用于重试）
    fn execute_for_retry(&self) -> Result<Response, HttpError> {
        let start = Instant::now();
        let request = self.build_request_ref()?;

        let response = request.send().map_err(|e| {
            let context = self.create_error_context(start.elapsed());
            self.convert_reqwest_error(e, context)
        })?;

        let duration = start.elapsed();

        Response::from_reqwest(
            response,
            self.method,
            duration,
            self.client.max_response_body_size,
        )
    }

    /// 构建请求（可变，消费 multipart）
    fn build_request_mut(&mut self) -> Result<reqwest::blocking::RequestBuilder, HttpError> {
        let mut request = match self.method {
            HttpMethod::GET => self.client.client.get(&self.url),
            HttpMethod::POST => self.client.client.post(&self.url),
            HttpMethod::PUT => self.client.client.put(&self.url),
            HttpMethod::DELETE => self.client.client.delete(&self.url),
            HttpMethod::PATCH => self.client.client.patch(&self.url),
            HttpMethod::HEAD => self.client.client.head(&self.url),
            HttpMethod::OPTIONS => self.client.client.request(reqwest::Method::OPTIONS, &self.url),
        };

        // 设置 multipart 或 body（multipart 优先）
        if let Some(form) = self.multipart.take() {
            request = request.multipart(form);
        } else if let Some(body) = &self.body {
            request = request.json(body);
        }

        // 设置查询参数
        if let Some(query) = &self.query {
            request = request.query(query);
        }

        // 设置请求头
        if let Some(headers) = &self.headers {
            for (key, value) in headers.iter() {
                request = request.header(key, value);
            }
        }

        // 设置认证
        if let Some(auth) = &self.auth {
            request = self.apply_auth(request, auth)?;
        }

        // 设置超时
        let timeout = self.timeout.unwrap_or(self.client.timeout);
        request = request.timeout(timeout);

        Ok(request)
    }

    /// 构建请求（引用，用于重试，不含 multipart）
    fn build_request_ref(&self) -> Result<reqwest::blocking::RequestBuilder, HttpError> {
        let mut request = match self.method {
            HttpMethod::GET => self.client.client.get(&self.url),
            HttpMethod::POST => self.client.client.post(&self.url),
            HttpMethod::PUT => self.client.client.put(&self.url),
            HttpMethod::DELETE => self.client.client.delete(&self.url),
            HttpMethod::PATCH => self.client.client.patch(&self.url),
            HttpMethod::HEAD => self.client.client.head(&self.url),
            HttpMethod::OPTIONS => self.client.client.request(reqwest::Method::OPTIONS, &self.url),
        };

        // 只设置 body（不含 multipart）
        if let Some(body) = &self.body {
            request = request.json(body);
        }

        // 设置查询参数
        if let Some(query) = &self.query {
            request = request.query(query);
        }

        // 设置请求头
        if let Some(headers) = &self.headers {
            for (key, value) in headers.iter() {
                request = request.header(key, value);
            }
        }

        // 设置认证
        if let Some(auth) = &self.auth {
            request = self.apply_auth(request, auth)?;
        }

        // 设置超时
        let timeout = self.timeout.unwrap_or(self.client.timeout);
        request = request.timeout(timeout);

        Ok(request)
    }

    /// 应用认证
    fn apply_auth(
        &self,
        request: reqwest::blocking::RequestBuilder,
        auth: &Authorization,
    ) -> Result<reqwest::blocking::RequestBuilder, HttpError> {
        match auth {
            Authorization::Basic { username, password } => {
                Ok(request.basic_auth(username, Some(password)))
            }
            Authorization::Bearer { token } => Ok(request.bearer_auth(token)),
            Authorization::Custom {
                header: _,
                value: _,
            } => {
                let mut headers = HeaderMap::new();
                auth.apply_to_headers(&mut headers).map_err(|e| HttpError::RequestBuild {
                    message: format!("Failed to apply auth header: {}", e),
                    context: ErrorContext::boxed(&self.url, self.method),
                })?;
                let mut req = request;
                for (k, v) in headers.iter() {
                    req = req.header(k, v);
                }
                Ok(req)
            }
        }
    }

    /// 创建错误上下文
    fn create_error_context(&self, duration: Duration) -> Box<ErrorContext> {
        ErrorContext::new(&self.url, self.method).with_duration(duration).into_box()
    }

    /// 转换 reqwest 错误
    fn convert_reqwest_error(
        &self,
        error: reqwest::Error,
        context: Box<ErrorContext>,
    ) -> HttpError {
        if error.is_timeout() {
            HttpError::Timeout { context }
        } else if error.is_connect() {
            HttpError::Connection { context }
        } else {
            HttpError::Request {
                source: error,
                context,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_chain() {
        let client = HttpClient::new().unwrap();
        let request = client
            .post("https://example.com/api")
            .body(&serde_json::json!({"name": "test"}))
            .header("X-Custom", "value")
            .auth(Authorization::bearer("token"))
            .timeout(Duration::from_secs(60));

        assert_eq!(request.url, "https://example.com/api");
        assert_eq!(request.method, HttpMethod::POST);
        assert!(request.body.is_some());
        assert!(request.headers.is_some());
        assert!(request.auth.is_some());
        assert_eq!(request.timeout, Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_request_query() {
        let client = HttpClient::new().unwrap();
        let request = client.get("https://example.com").query(&[("page", "1")]);
        assert!(request.query.is_some());
    }

    #[test]
    fn test_request_retry() {
        let client = HttpClient::new().unwrap();
        let request = client.get("https://example.com").retry(RetryConfig::new().max_retries(5));
        assert!(request.retry.is_some());
        assert_eq!(request.retry.unwrap().max_retries, 5);
    }

    #[test]
    fn test_request_multipart() {
        let client = HttpClient::new().unwrap();
        let request = client
            .post("https://example.com/upload")
            .multipart(MultipartRequest::new().text("field", "value"));
        assert!(request.multipart.is_some());
    }
}
