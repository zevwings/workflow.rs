//! HTTP 方法枚举

use std::fmt;
use std::str::FromStr;

/// HTTP 请求方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HttpMethod {
    /// GET 请求
    #[default]
    Get,
    /// POST 请求
    Post,
    /// PUT 请求
    Put,
    /// DELETE 请求
    Delete,
    /// PATCH 请求
    Patch,
    /// HEAD 请求
    Head,
    /// OPTIONS 请求
    Options,
}

impl HttpMethod {
    /// 是否是幂等方法
    pub fn is_idempotent(self) -> bool {
        matches!(
            self,
            Self::Get | Self::Put | Self::Delete | Self::Head | Self::Options
        )
    }

    /// 是否可以有请求体
    pub fn can_have_body(self) -> bool {
        matches!(self, Self::Post | Self::Put | Self::Patch)
    }
}

impl From<HttpMethod> for reqwest::Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Patch => reqwest::Method::PATCH,
            HttpMethod::Head => reqwest::Method::HEAD,
            HttpMethod::Options => reqwest::Method::OPTIONS,
        }
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
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
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "PATCH" => Ok(Self::Patch),
            "HEAD" => Ok(Self::Head),
            "OPTIONS" => Ok(Self::Options),
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
        assert_eq!(HttpMethod::Get.to_string(), "GET");
        assert_eq!(HttpMethod::Post.to_string(), "POST");
        assert_eq!(HttpMethod::Put.to_string(), "PUT");
        assert_eq!(HttpMethod::Delete.to_string(), "DELETE");
        assert_eq!(HttpMethod::Patch.to_string(), "PATCH");
    }

    #[test]
    fn test_from_str() {
        assert_eq!("get".parse::<HttpMethod>().unwrap(), HttpMethod::Get);
        assert_eq!("POST".parse::<HttpMethod>().unwrap(), HttpMethod::Post);
        assert_eq!("Put".parse::<HttpMethod>().unwrap(), HttpMethod::Put);
    }

    #[test]
    fn test_from_str_invalid() {
        assert!("INVALID".parse::<HttpMethod>().is_err());
    }

    #[test]
    fn test_is_idempotent() {
        assert!(HttpMethod::Get.is_idempotent());
        assert!(HttpMethod::Put.is_idempotent());
        assert!(HttpMethod::Delete.is_idempotent());
        assert!(!HttpMethod::Post.is_idempotent());
        assert!(!HttpMethod::Patch.is_idempotent());
    }

    #[test]
    fn test_can_have_body() {
        assert!(HttpMethod::Post.can_have_body());
        assert!(HttpMethod::Put.can_have_body());
        assert!(HttpMethod::Patch.can_have_body());
        assert!(!HttpMethod::Get.can_have_body());
        assert!(!HttpMethod::Delete.can_have_body());
    }
}
