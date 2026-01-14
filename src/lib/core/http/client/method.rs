//! HTTP 方法定义

use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// HTTP 方法解析错误
#[derive(Debug, Error)]
pub enum HttpMethodError {
    /// 无效的 HTTP 方法
    #[error("Invalid HTTP method: {0}")]
    InvalidMethod(String),
}

/// HTTP 方法枚举
#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl FromStr for HttpMethod {
    type Err = HttpMethodError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "PATCH" => Ok(HttpMethod::Patch),
            _ => Err(HttpMethodError::InvalidMethod(s.to_string())),
        }
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
        }
    }
}
