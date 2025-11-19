//! HTTP 方法枚举

use anyhow::Result;
use std::fmt;
use std::str::FromStr;

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
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "PATCH" => Ok(HttpMethod::Patch),
            _ => anyhow::bail!("Invalid HTTP method: {}", s),
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
