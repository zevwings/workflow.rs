//! Basic Authentication 认证信息

/// Basic Authentication 认证信息
///
/// 用于 HTTP Basic Authentication 的用户名和密码。
#[derive(Debug, Clone)]
pub struct Authorization {
    /// 用户名（通常是邮箱地址）
    pub username: String,
    /// 密码（通常是 API token）
    pub password: String,
}

impl Authorization {
    /// 创建新的 Authorization
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
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_basic() {
        // Basic validation that Authorization can be created
        let auth = Authorization::new("user@example.com", "api_token");
        assert_eq!(auth.username, "user@example.com");
        assert_eq!(auth.password, "api_token");
    }
}
