//! Codeup 实体类型

use serde::{Deserialize, Serialize};

/// Codeup 用户信息
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeupUser {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
}

impl CodeupUser {
    /// 创建新的 Codeup 用户
    pub fn new(id: i64, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            email: None,
        }
    }

    /// 设置用户邮箱
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codeup_user_new() {
        let user = CodeupUser::new(123, "test_user");
        assert_eq!(user.id, 123);
        assert_eq!(user.name, "test_user");
        assert_eq!(user.email, None);
    }

    #[test]
    fn test_codeup_user_with_email() {
        let user = CodeupUser::new(123, "test_user").with_email("test@example.com");
        assert_eq!(user.email, Some("test@example.com".to_string()));
    }
}
