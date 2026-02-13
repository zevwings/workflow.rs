//! ReqwestHttpClient：用 reqwest 实现 client::HttpClient

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use reqwest::{
    blocking::Client as ReqwestClient,
    header::{HeaderMap, HeaderName, HeaderValue},
    Method as ReqwestMethod,
};

use client::{
    Authorization, HttpClient, HttpClientConfig, HttpError, HttpMethod, HttpRequest, HttpResponse,
};
use toolkit::log_error;

use super::{auth, error, response};

fn to_reqwest_method(m: HttpMethod) -> ReqwestMethod {
    match m {
        HttpMethod::GET => ReqwestMethod::GET,
        HttpMethod::POST => ReqwestMethod::POST,
        HttpMethod::PUT => ReqwestMethod::PUT,
        HttpMethod::DELETE => ReqwestMethod::DELETE,
        HttpMethod::PATCH => ReqwestMethod::PATCH,
        HttpMethod::HEAD => ReqwestMethod::HEAD,
        HttpMethod::OPTIONS => ReqwestMethod::OPTIONS,
    }
}

fn build_headers(
    headers: &HashMap<String, String>,
    auth: Option<&Authorization>,
) -> Result<HeaderMap, HttpError> {
    let mut map = HeaderMap::new();
    for (k, v) in headers {
        let name = HeaderName::try_from(k.as_str())
            .map_err(|e| HttpError::InvalidHeaderName(e.to_string()))?;
        let value =
            HeaderValue::from_str(v).map_err(|e| HttpError::InvalidHeaderValue(e.to_string()))?;
        map.insert(name, value);
    }
    if let Some(a) = auth {
        auth::apply_auth_to_headers(&mut map, a)?;
    }
    Ok(map)
}

/// 用 reqwest 实现的 HTTP 客户端
///
/// 持有 `reqwest::blocking::Client`，通过 DI 注入 `dyn client::HttpClient` 或直接构造使用。
///
/// # 示例
///
/// ```rust,ignore
/// use infra::http::ReqwestHttpClient;
/// use client::HttpClientConfig;
///
/// let client = ReqwestHttpClient::with_config(HttpClientConfig::new().base_url("https://api.example.com"))?;
/// let response = client.get("/users").send()?;
/// ```
pub struct ReqwestHttpClient {
    client: ReqwestClient,
    base_url: Option<String>,
    timeout: Duration,
    max_response_body_size: usize,
}

impl ReqwestHttpClient {
    /// 使用默认配置创建客户端
    pub fn new() -> Result<Self, HttpError> {
        Self::with_config(HttpClientConfig::default())
    }

    /// 使用自定义配置创建客户端
    pub fn with_config(config: HttpClientConfig) -> Result<Self, HttpError> {
        let mut builder = ReqwestClient::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.timeout)
            .user_agent(&config.user_agent);

        if !config.tls_verify {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let client = builder.build().map_err(|e| {
            log_error!(error = %e, "Failed to create HTTP client");
            HttpError::ClientCreation(e.to_string())
        })?;

        Ok(Self {
            client,
            base_url: config.base_url,
            timeout: config.timeout,
            max_response_body_size: config.max_response_body_size,
        })
    }
}

impl HttpClient for ReqwestHttpClient {
    fn base_url(&self) -> Option<&str> {
        self.base_url.as_deref()
    }

    fn execute(&self, req: HttpRequest) -> Result<HttpResponse, HttpError> {
        let start = Instant::now();
        let url = req.url.clone();
        let method = req.method;

        let reqwest_method = to_reqwest_method(req.method);

        let mut request = self
            .client
            .request(reqwest_method, &url)
            .timeout(req.timeout.unwrap_or(self.timeout));

        // 设置 body
        if let Some(body) = &req.body {
            request = request.json(body);
        }

        // 设置 query
        if let Some(query) = &req.query {
            request = request.query(query);
        }

        let headers = build_headers(&req.headers, req.auth.as_ref())?;
        for (k, v) in headers.iter() {
            request = request.header(k, v);
        }

        let response = request
            .send()
            .map_err(|e| error::from_reqwest(e, &url, method, start.elapsed()))?;

        let duration = start.elapsed();

        // 非 2xx 不在这里转错误，由调用方决定是否 ensure_success
        response::from_reqwest(response, method, duration, self.max_response_body_size)
    }

    fn execute_multipart(
        &self,
        method: HttpMethod,
        url: &str,
        multipart: client::MultipartRequest,
        query: Option<serde_json::Value>,
        headers: HashMap<String, String>,
        auth: Option<Authorization>,
        timeout: Option<Duration>,
    ) -> Result<HttpResponse, HttpError> {
        let form = super::multipart::to_reqwest_form(multipart)?;
        let start = Instant::now();

        let reqwest_method = to_reqwest_method(method);

        let mut request = self
            .client
            .request(reqwest_method, url)
            .multipart(form)
            .timeout(timeout.unwrap_or(self.timeout));

        if let Some(q) = &query {
            request = request.query(q);
        }

        let header_map = build_headers(&headers, auth.as_ref())?;
        for (k, v) in header_map.iter() {
            request = request.header(k, v);
        }

        let response = request
            .send()
            .map_err(|e| error::from_reqwest(e, url, method, start.elapsed()))?;

        let duration = start.elapsed();
        response::from_reqwest(response, method, duration, self.max_response_body_size)
    }
}
