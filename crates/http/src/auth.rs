//! HTTP 认证信息

use std::str::FromStr;

use base64::{engine::general_purpose, Engine};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};

use crate::HttpError;

/// HTTP 认证信息
///
/// 支持多种认证方式：
/// - Basic Authentication（用户名和密码）
/// - Bearer Token（API token）
/// - 自定义 Header 作为认证
#[derive(Debug, Clone)]
pub enum Authorization {
    /// Basic Authentication（用户名和密码）
    Basic {
        /// 用户名（通常是邮箱地址）
        username: String,
        /// 密码（通常是 API token）
        password: String,
    },
    /// Bearer Token 认证
    Bearer {
        /// Bearer Token
        token: String,
    },
    /// 自定义 Header 作为认证
    Custom {
        /// Header 名称（例如 X-Api-Key）
        header: String,
        /// Header 值
        value: String,
    },
}

impl Authorization {
    /// 创建 Basic Authentication
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::Basic {
            username: username.into(),
            password: password.into(),
        }
    }

    /// 创建 Bearer Token 认证
    pub fn bearer(token: impl Into<String>) -> Self {
        Self::Bearer {
            token: token.into(),
        }
    }

    /// 创建自定义认证 Header
    pub fn custom(header: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Custom {
            header: header.into(),
            value: value.into(),
        }
    }

    /// 将认证信息应用到 HTTP Headers
    pub fn apply_to_headers(&self, headers: &mut HeaderMap) -> Result<(), HttpError> {
        match self {
            Self::Basic { username, password } => {
                let credentials = format!("{}:{}", username, password);
                let encoded = general_purpose::STANDARD.encode(credentials.as_bytes());
                let value = format!("Basic {}", encoded);
                let header_value = HeaderValue::from_str(&value).map_err(|e| {
                    error!("Invalid header value: {}", e);
                    HttpError::InvalidHeaderValue(e.to_string())
                })?;
                headers.insert(AUTHORIZATION, header_value);
            }
            Self::Bearer { token } => {
                let value = format!("Bearer {}", token);
                let header_value = HeaderValue::from_str(&value).map_err(|e| {
                    error!("Invalid header value: {}", e);
                    HttpError::InvalidHeaderValue(e.to_string())
                })?;
                headers.insert(AUTHORIZATION, header_value);
            }
            Self::Custom { header, value } => {
                let header_name = HeaderName::from_str(header).map_err(|e| {
                    error!("Invalid header name: {}", e);
                    HttpError::InvalidHeaderName(e.to_string())
                })?;
                let header_value = HeaderValue::from_str(value).map_err(|e| {
                    error!("Invalid header value: {}", e);
                    HttpError::InvalidHeaderValue(e.to_string())
                })?;
                headers.insert(header_name, header_value);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_auth_apply_to_headers() {
        let mut headers = HeaderMap::new();
        let auth = Authorization::basic("user@example.com", "api_token");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_ok());
        let auth_header = headers.get(AUTHORIZATION).unwrap();
        let auth_value = auth_header.to_str().unwrap();
        assert!(auth_value.starts_with("Basic "));
    }

    #[test]
    fn test_bearer_auth_apply_to_headers() {
        let mut headers = HeaderMap::new();
        let auth = Authorization::bearer("your_api_token");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_ok());
        let auth_header = headers.get(AUTHORIZATION).unwrap();
        let auth_value = auth_header.to_str().unwrap();
        assert_eq!(auth_value, "Bearer your_api_token");
    }

    #[test]
    fn test_custom_auth_apply_to_headers() {
        let mut headers = HeaderMap::new();
        let auth = Authorization::custom("X-Api-Key", "api_key_value");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_ok());
        let custom_header = headers.get("X-Api-Key").unwrap();
        let header_value = custom_header.to_str().unwrap();
        assert_eq!(header_value, "api_key_value");
    }
}
