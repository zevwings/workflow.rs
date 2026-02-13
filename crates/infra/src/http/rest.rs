//! REST API 客户端辅助工具
//!
//! 提供通用的 REST API 客户端构建器，减少样板代码。

use std::collections::HashMap;

use client::{Authorization, HttpClientHolder, HttpError, HttpMethod, HttpRequest, HttpResponse};

/// REST API 请求构建器
///
/// 提供流畅的 API 来构建和执行 REST 请求，减少样板代码。
///
/// # 示例
///
/// ```rust,ignore
/// let response = RestRequestBuilder::new(&holder, HttpMethod::POST, "/api/users")
///     .headers(headers)
///     .auth(auth)
///     .body(body)
///     .query(query)
///     .execute()?;
/// ```
pub struct RestRequestBuilder<'a> {
    holder: &'a HttpClientHolder,
    method: HttpMethod,
    url: String,
    headers: HashMap<String, String>,
    auth: Option<Authorization>,
    body: Option<serde_json::Value>,
    query: Option<serde_json::Value>,
}

impl<'a> RestRequestBuilder<'a> {
    /// 创建新的请求构建器
    pub fn new(holder: &'a HttpClientHolder, method: HttpMethod, url: impl Into<String>) -> Self {
        Self {
            holder,
            method,
            url: url.into(),
            headers: HashMap::new(),
            auth: None,
            body: None,
            query: None,
        }
    }

    /// 设置请求头
    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    /// 设置认证
    pub fn auth(mut self, auth: Authorization) -> Self {
        self.auth = Some(auth);
        self
    }

    /// 设置请求体
    pub fn body(mut self, body: Option<serde_json::Value>) -> Self {
        self.body = body;
        self
    }

    /// 设置查询参数
    pub fn query(mut self, query: Option<serde_json::Value>) -> Self {
        self.query = query;
        self
    }

    /// 执行请求
    pub fn execute(self) -> Result<HttpResponse, HttpError> {
        let http_request = HttpRequest {
            method: self.method,
            url: self.url,
            headers: self.headers,
            query: self.query,
            body: self.body,
            auth: self.auth,
            timeout: None,
        };

        self.holder.execute(http_request)
    }
}
