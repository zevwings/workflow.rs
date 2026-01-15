//! HTTP 认证信息

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use std::str::FromStr;

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
    /// 创建新的 Basic Authentication
    ///
    /// 创建 Basic Authentication 认证信息。
    ///
    /// # 参数
    ///
    /// * `username` - 用户名（通常是邮箱地址）
    /// * `password` - 密码（通常是 API token）
    ///
    /// # 返回
    ///
    /// 返回 `Authorization` 结构体。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::Authorization;
    ///
    /// let auth = Authorization::basic("user@example.com", "api_token");
    /// ```
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::Basic {
            username: username.into(),
            password: password.into(),
        }
    }

    /// 创建新的 Bearer Token 认证
    ///
    /// 创建 Bearer Token 认证信息。
    ///
    /// # 参数
    ///
    /// * `token` - Bearer Token
    ///
    /// # 返回
    ///
    /// 返回 `Authorization` 结构体。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::Authorization;
    ///
    /// let auth = Authorization::bearer("your_api_token");
    /// ```
    pub fn bearer(token: impl Into<String>) -> Self {
        Self::Bearer {
            token: token.into(),
        }
    }

    /// 创建自定义认证 Header
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::Authorization;
    ///
    /// let auth = Authorization::custom("X-Api-Key", "token");
    /// ```
    pub fn custom(header: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Custom {
            header: header.into(),
            value: value.into(),
        }
    }

    /// 将认证信息应用到 HTTP Headers
    ///
    /// 根据认证类型，将相应的 Authorization header 添加到 HeaderMap 中。
    ///
    /// # 参数
    ///
    /// * `headers` - 要添加认证信息的 HeaderMap
    ///
    /// # 错误
    ///
    /// 如果无法解析 header 值，返回错误。
    pub fn apply_to_headers(
        &self,
        headers: &mut HeaderMap,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self {
            Self::Basic { username, password } => {
                use base64::engine::general_purpose;
                use base64::Engine;
                let credentials = format!("{}:{}", username, password);
                let encoded = general_purpose::STANDARD.encode(credentials.as_bytes());
                let value = format!("Basic {}", encoded);
                headers.insert(AUTHORIZATION, HeaderValue::from_str(&value)?);
            }
            Self::Bearer { token } => {
                let value = format!("Bearer {}", token);
                headers.insert(AUTHORIZATION, HeaderValue::from_str(&value)?);
            }
            Self::Custom { header, value } => {
                let header_name = HeaderName::from_str(header)?;
                headers.insert(header_name, HeaderValue::from_str(value)?);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, AUTHORIZATION};

    // ==================== Basic Authentication 测试 ====================

    #[test]
    fn test_basic_auth_apply_to_headers() {
        let mut headers = HeaderMap::new();
        let auth = Authorization::basic("user@example.com", "api_token");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_ok());
        let auth_header = headers.get(AUTHORIZATION);
        assert!(auth_header.is_some());
        let auth_value = auth_header.unwrap().to_str().unwrap();
        assert!(auth_value.starts_with("Basic "));
        // 验证 Base64 编码（user@example.com:api_token 的 Base64）
        use base64::engine::general_purpose;
        use base64::Engine;
        let encoded = auth_value.strip_prefix("Basic ").unwrap();
        let decoded = general_purpose::STANDARD.decode(encoded).unwrap();
        let credentials = String::from_utf8(decoded).unwrap();
        assert_eq!(credentials, "user@example.com:api_token");
    }

    #[test]
    fn test_basic_auth_with_special_characters() {
        let mut headers = HeaderMap::new();
        let auth = Authorization::basic("user:name", "pass:word");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_ok());
        let auth_header = headers.get(AUTHORIZATION);
        assert!(auth_header.is_some());
        let auth_value = auth_header.unwrap().to_str().unwrap();
        assert!(auth_value.starts_with("Basic "));
    }

    // ==================== Bearer Token 测试 ====================

    #[test]
    fn test_bearer_auth_apply_to_headers() {
        let mut headers = HeaderMap::new();
        let auth = Authorization::bearer("your_api_token");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_ok());
        let auth_header = headers.get(AUTHORIZATION);
        assert!(auth_header.is_some());
        let auth_value = auth_header.unwrap().to_str().unwrap();
        assert_eq!(auth_value, "Bearer your_api_token");
    }

    #[test]
    fn test_bearer_auth_with_special_characters() {
        let mut headers = HeaderMap::new();
        let auth = Authorization::bearer("token-with-special-chars!@#$%");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_ok());
        let auth_header = headers.get(AUTHORIZATION);
        assert!(auth_header.is_some());
        let auth_value = auth_header.unwrap().to_str().unwrap();
        assert_eq!(auth_value, "Bearer token-with-special-chars!@#$%");
    }

    // ==================== Custom Header 测试 ====================

    #[test]
    fn test_custom_auth_apply_to_headers() {
        let mut headers = HeaderMap::new();
        let auth = Authorization::custom("X-Api-Key", "api_key_value");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_ok());
        let custom_header = headers.get("X-Api-Key");
        assert!(custom_header.is_some());
        let header_value = custom_header.unwrap().to_str().unwrap();
        assert_eq!(header_value, "api_key_value");
    }

    #[test]
    fn test_custom_auth_with_different_header_name() {
        let mut headers = HeaderMap::new();
        let auth = Authorization::custom("X-Custom-Auth", "custom_value");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_ok());
        let custom_header = headers.get("X-Custom-Auth");
        assert!(custom_header.is_some());
        let header_value = custom_header.unwrap().to_str().unwrap();
        assert_eq!(header_value, "custom_value");
    }

    // ==================== 错误处理测试 ====================

    #[test]
    fn test_custom_auth_invalid_header_name() {
        let mut headers = HeaderMap::new();
        // 使用包含非法字符的 header 名（如包含换行符）
        let auth = Authorization::custom("Invalid\nHeader", "value");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_err());
    }

    #[test]
    fn test_custom_auth_invalid_header_value() {
        let mut headers = HeaderMap::new();
        // 使用包含非法字符的 header 值（如包含控制字符）
        let auth = Authorization::custom("X-Api-Key", "value\x00invalid");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_err());
    }

    #[test]
    fn test_custom_auth_empty_header_name() {
        let mut headers = HeaderMap::new();
        let auth = Authorization::custom("", "value");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_err());
    }

    // ==================== 边界条件测试 ====================

    #[test]
    fn test_basic_auth_empty_username() {
        let mut headers = HeaderMap::new();
        let auth = Authorization::basic("", "password");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_ok());
        let auth_header = headers.get(AUTHORIZATION);
        assert!(auth_header.is_some());
    }

    #[test]
    fn test_basic_auth_empty_password() {
        let mut headers = HeaderMap::new();
        let auth = Authorization::basic("username", "");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_ok());
        let auth_header = headers.get(AUTHORIZATION);
        assert!(auth_header.is_some());
    }

    #[test]
    fn test_bearer_auth_empty_token() {
        let mut headers = HeaderMap::new();
        let auth = Authorization::bearer("");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_ok());
        let auth_header = headers.get(AUTHORIZATION);
        assert!(auth_header.is_some());
        let auth_value = auth_header.unwrap().to_str().unwrap();
        assert_eq!(auth_value, "Bearer ");
    }

    #[test]
    fn test_custom_auth_empty_value() {
        let mut headers = HeaderMap::new();
        let auth = Authorization::custom("X-Api-Key", "");

        let result = auth.apply_to_headers(&mut headers);

        assert!(result.is_ok());
        let custom_header = headers.get("X-Api-Key");
        assert!(custom_header.is_some());
        let header_value = custom_header.unwrap().to_str().unwrap();
        assert_eq!(header_value, "");
    }

    // ==================== 多次应用测试 ====================

    #[test]
    fn test_apply_multiple_auth_types() {
        // 测试不同类型的认证不会互相干扰
        let mut headers1 = HeaderMap::new();
        let basic_auth = Authorization::basic("user", "pass");
        assert!(basic_auth.apply_to_headers(&mut headers1).is_ok());

        let mut headers2 = HeaderMap::new();
        let bearer_auth = Authorization::bearer("token");
        assert!(bearer_auth.apply_to_headers(&mut headers2).is_ok());

        let mut headers3 = HeaderMap::new();
        let custom_auth = Authorization::custom("X-Api-Key", "key");
        assert!(custom_auth.apply_to_headers(&mut headers3).is_ok());

        // 验证每个都正确设置
        assert!(headers1.get(AUTHORIZATION).is_some());
        assert!(headers2.get(AUTHORIZATION).is_some());
        assert!(headers3.get("X-Api-Key").is_some());
    }
}
