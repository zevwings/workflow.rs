//! HTTP 方法枚举

use std::fmt;
use std::str::FromStr;

use reqwest::Method as ReqwestMethod;

/// HTTP 请求方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HttpMethod {
    /// GET 请求
    #[default]
    GET,
    /// POST 请求
    POST,
    /// PUT 请求
    PUT,
    /// DELETE 请求
    DELETE,
    /// PATCH 请求
    PATCH,
    /// HEAD 请求
    HEAD,
    /// OPTIONS 请求
    OPTIONS,
}

impl HttpMethod {
    /// 是否是幂等方法
    pub fn is_idempotent(self) -> bool {
        matches!(
            self,
            Self::GET | Self::PUT | Self::DELETE | Self::HEAD | Self::OPTIONS
        )
    }

    /// 是否可以有请求体
    pub fn can_have_body(self) -> bool {
        matches!(self, Self::POST | Self::PUT | Self::PATCH)
    }
}

impl From<HttpMethod> for ReqwestMethod {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::GET => ReqwestMethod::GET,
            HttpMethod::POST => ReqwestMethod::POST,
            HttpMethod::PUT => ReqwestMethod::PUT,
            HttpMethod::DELETE => ReqwestMethod::DELETE,
            HttpMethod::PATCH => ReqwestMethod::PATCH,
            HttpMethod::HEAD => ReqwestMethod::HEAD,
            HttpMethod::OPTIONS => ReqwestMethod::OPTIONS,
        }
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::GET => "GET",
            Self::POST => "POST",
            Self::PUT => "PUT",
            Self::DELETE => "DELETE",
            Self::PATCH => "PATCH",
            Self::HEAD => "HEAD",
            Self::OPTIONS => "OPTIONS",
        };
        write!(f, "{}", s)
    }
}

/// HTTP 方法解析错误
#[derive(Debug, Clone)]
pub struct ParseMethodError {
    method: String,
}

impl fmt::Display for ParseMethodError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid HTTP method: {}", self.method)
    }
}

impl std::error::Error for ParseMethodError {}

impl FromStr for HttpMethod {
    type Err = ParseMethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "PATCH" => Ok(Self::PATCH),
            "HEAD" => Ok(Self::HEAD),
            "OPTIONS" => Ok(Self::OPTIONS),
            _ => Err(ParseMethodError {
                method: s.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(HttpMethod::GET.to_string(), "GET");
        assert_eq!(HttpMethod::POST.to_string(), "POST");
        assert_eq!(HttpMethod::PUT.to_string(), "PUT");
        assert_eq!(HttpMethod::DELETE.to_string(), "DELETE");
        assert_eq!(HttpMethod::PATCH.to_string(), "PATCH");
    }

    #[test]
    fn test_from_str() {
        assert_eq!("get".parse::<HttpMethod>().unwrap(), HttpMethod::GET);
        assert_eq!("POST".parse::<HttpMethod>().unwrap(), HttpMethod::POST);
        assert_eq!("Put".parse::<HttpMethod>().unwrap(), HttpMethod::PUT);
    }

    #[test]
    fn test_from_str_invalid() {
        assert!("INVALID".parse::<HttpMethod>().is_err());
    }

    #[test]
    fn test_is_idempotent() {
        assert!(HttpMethod::GET.is_idempotent());
        assert!(HttpMethod::PUT.is_idempotent());
        assert!(HttpMethod::DELETE.is_idempotent());
        assert!(!HttpMethod::POST.is_idempotent());
        assert!(!HttpMethod::PATCH.is_idempotent());
    }

    #[test]
    fn test_can_have_body() {
        assert!(HttpMethod::POST.can_have_body());
        assert!(HttpMethod::PUT.can_have_body());
        assert!(HttpMethod::PATCH.can_have_body());
        assert!(!HttpMethod::GET.can_have_body());
        assert!(!HttpMethod::DELETE.can_have_body());
    }
}
