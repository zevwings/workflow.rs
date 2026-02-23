//! 将 reqwest::Error 转换为 client::HttpError
//!
//! 实现层不持有 reqwest 类型，仅提取 message 字符串。

use std::time::Duration;

use reqwest::Error as ReqwestError;

use client::{ErrorContext, HttpError, HttpMethod};

/// 将 reqwest 错误转换为 HttpError
///
/// 提取错误信息为字符串，不持有 reqwest::Error。
/// 映射规则：超时 → `Timeout`；连接失败 → `Connection`；其他 → `Request`。
pub fn from_reqwest(
    error: ReqwestError,
    url: &str,
    method: HttpMethod,
    duration: Duration,
) -> HttpError {
    let context = ErrorContext::new(url, method).with_duration(duration).into_box();

    if error.is_timeout() {
        HttpError::Timeout { context }
    } else if error.is_connect() {
        HttpError::Connection { context }
    } else {
        HttpError::Request {
            message: error.to_string(),
            context,
        }
    }
}
