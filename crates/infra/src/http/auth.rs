//! 将 `Authorization` 转换为 reqwest 请求头
//!
//! 实现层负责将 client 定义的 `Authorization` 纯数据转换为实际 HTTP 头。

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};

use client::{Authorization, HttpError};

/// 将认证信息应用到 reqwest HeaderMap
///
/// 将 client 定义的 `Authorization` 转为 HTTP 头并插入 `headers`。
/// 无效的 header 名称或值会返回 `HttpError`。
pub fn apply_auth_to_headers(
    headers: &mut HeaderMap,
    auth: &Authorization,
) -> Result<(), HttpError> {
    match auth {
        Authorization::Basic { username, password } => {
            let credentials = format!("{}:{}", username, password);
            let encoded = BASE64.encode(credentials.as_bytes());
            let value = format!("Basic {}", encoded);
            let header_value = HeaderValue::from_str(&value)
                .map_err(|e| HttpError::InvalidHeaderValue(e.to_string()))?;
            headers.insert(AUTHORIZATION, header_value);
        }
        Authorization::Bearer { token } => {
            let value = format!("Bearer {}", token);
            let header_value = HeaderValue::from_str(&value)
                .map_err(|e| HttpError::InvalidHeaderValue(e.to_string()))?;
            headers.insert(AUTHORIZATION, header_value);
        }
        Authorization::Custom { header, value } => {
            let header_name = HeaderName::try_from(header.as_str())
                .map_err(|e| HttpError::InvalidHeaderName(e.to_string()))?;
            let header_value = HeaderValue::from_str(value)
                .map_err(|e| HttpError::InvalidHeaderValue(e.to_string()))?;
            headers.insert(header_name, header_value);
        }
    }
    Ok(())
}
