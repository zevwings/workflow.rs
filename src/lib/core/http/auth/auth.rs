//! HTTP 认证信息

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

/// HTTP 认证信息
///
/// 支持多种认证方式：
/// - Basic Authentication（用户名和密码）
/// - Bearer Token（API token）
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
    ) -> Result<(), reqwest::header::InvalidHeaderValue> {
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
        }
        Ok(())
    }
}
