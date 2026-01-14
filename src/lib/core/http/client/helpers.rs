//! HTTP 客户端辅助函数

use reqwest::blocking::RequestBuilder;
use reqwest::header::HeaderMap;
use std::time::Duration;

use crate::core::http::auth::Authorization;

/// 应用通用请求配置（query, auth, headers, timeout）
pub(crate) fn apply_common_config(
    mut request: RequestBuilder,
    query: &Option<serde_json::Value>,
    auth: &Option<Authorization>,
    headers: &Option<HeaderMap>,
    timeout: Option<Duration>,
) -> RequestBuilder {
    // 添加 query 参数
    if let Some(query) = query {
        request = request.query(query);
    }

    // 添加 auth
    if let Some(auth) = auth {
        match auth {
            Authorization::Basic { username, password } => {
                request = request.basic_auth(username, Some(password));
            }
            Authorization::Bearer { token } => {
                request = request.bearer_auth(token);
            }
        }
    }

    // 添加 headers
    if let Some(headers) = headers {
        for (key, value) in headers.iter() {
            request = request.header(key, value);
        }
    }

    // 设置超时（如果提供了则使用，否则使用默认 30 秒）
    let timeout_duration = timeout.unwrap_or_else(|| Duration::from_secs(30));
    request = request.timeout(timeout_duration);

    request
}

/// 为 multipart 请求应用认证（Bearer token 需要特殊处理）
pub(crate) fn apply_auth_for_multipart(
    mut request: RequestBuilder,
    auth: &Authorization,
) -> RequestBuilder {
    match auth {
        Authorization::Basic { username, password } => {
            request = request.basic_auth(username, Some(password));
        }
        Authorization::Bearer { token: _ } => {
            let mut auth_headers = HeaderMap::new();
            if auth.apply_to_headers(&mut auth_headers).is_ok() {
                for (key, value) in auth_headers.iter() {
                    request = request.header(key, value);
                }
            }
        }
    }
    request
}
