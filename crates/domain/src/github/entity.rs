//! GitHub 实体类型

/// GitHub 用户信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubUser {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

impl GitHubUser {
    /// 创建新的 GitHub 用户
    pub fn new(login: impl Into<String>) -> Self {
        Self {
            login: login.into(),
            name: None,
            email: None,
        }
    }

    /// 设置用户名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
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
    fn test_github_user_new() {
        let user = GitHubUser::new("octocat");
        assert_eq!(user.login, "octocat");
        assert_eq!(user.name, None);
        assert_eq!(user.email, None);
    }

    #[test]
    fn test_github_user_with_name() {
        let user = GitHubUser::new("octocat").with_name("The Octocat");
        assert_eq!(user.login, "octocat");
        assert_eq!(user.name, Some("The Octocat".to_string()));
    }

    #[test]
    fn test_github_user_with_email() {
        let user = GitHubUser::new("octocat").with_email("octocat@github.com");
        assert_eq!(user.login, "octocat");
        assert_eq!(user.email, Some("octocat@github.com".to_string()));
    }

    #[test]
    fn test_github_user_builder_chain() {
        let user = GitHubUser::new("developer")
            .with_name("John Doe")
            .with_email("john@example.com");

        assert_eq!(user.login, "developer");
        assert_eq!(user.name, Some("John Doe".to_string()));
        assert_eq!(user.email, Some("john@example.com".to_string()));
    }

    #[test]
    fn test_github_user_clone() {
        let user = GitHubUser::new("original")
            .with_name("Original User")
            .with_email("original@example.com");

        let cloned = user.clone();
        assert_eq!(user.login, cloned.login);
        assert_eq!(user.name, cloned.name);
        assert_eq!(user.email, cloned.email);
    }

    #[test]
    fn test_github_user_equality() {
        let user1 = GitHubUser::new("test").with_name("Test User");
        let user2 = GitHubUser::new("test").with_name("Test User");
        let user3 = GitHubUser::new("other").with_name("Test User");

        assert_eq!(user1, user2);
        assert_ne!(user1, user3);
    }

    #[test]
    fn test_github_user_debug() {
        let user = GitHubUser::new("debug_test");
        let debug_str = format!("{:?}", user);
        assert!(debug_str.contains("debug_test"));
        assert!(debug_str.contains("GitHubUser"));
    }

    #[test]
    fn test_github_user_with_string_type() {
        let user = GitHubUser::new(String::from("string_login"))
            .with_name(String::from("String Name"))
            .with_email(String::from("string@example.com"));

        assert_eq!(user.login, "string_login");
        assert_eq!(user.name, Some("String Name".to_string()));
        assert_eq!(user.email, Some("string@example.com".to_string()));
    }
}
