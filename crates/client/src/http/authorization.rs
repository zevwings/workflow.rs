//! HTTP 认证信息（定义层）
//!
//! 纯数据枚举，不依赖具体 HTTP 实现。
//! 实现层负责将 `Authorization` 转换为实际请求头。

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
}
