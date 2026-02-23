//! 将 reqwest::Response 转换为 client::HttpResponse

use std::{collections::HashMap, time::Duration};

use reqwest::blocking::Response as ReqwestResponse;

use client::{ErrorContext, HttpError, HttpMethod, HttpResponse};
use toolkit::log_error;

/// 从 reqwest 响应创建 client::HttpResponse
///
/// # 错误
///
/// - 读取响应体失败时返回 `HttpError::ResponseParse`
/// - 响应体超过 max_body_size 时返回 `HttpError::ResponseParse`
pub fn from_reqwest(
    response: ReqwestResponse,
    method: HttpMethod,
    duration: Duration,
    max_body_size: usize,
) -> Result<HttpResponse, HttpError> {
    let status = response.status().as_u16();
    let url = response.url().to_string();

    let headers: HashMap<String, String> = response
        .headers()
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|s| (k.as_str().to_lowercase(), s.to_string())))
        .collect();

    let body = response.bytes().map_err(|e| {
        log_error!(error = %e, "Failed to read response body");
        HttpError::ResponseParse {
            message: format!("Failed to read response body: {}", e),
            context: ErrorContext::new(&url, method)
                .with_response_status(status)
                .with_response_headers(headers.clone())
                .into_box(),
        }
    })?;

    let body = body.to_vec();

    if body.len() > max_body_size {
        return Err(HttpError::ResponseParse {
            message: format!(
                "Response body too large: {} bytes (max: {} bytes)",
                body.len(),
                max_body_size
            ),
            context: ErrorContext::new(&url, method)
                .with_response_status(status)
                .with_response_headers(headers)
                .into_box(),
        });
    }

    Ok(HttpResponse {
        status,
        headers,
        body,
        url,
        method,
        duration,
    })
}
