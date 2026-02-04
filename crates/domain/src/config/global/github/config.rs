//! GitHub 配置相关结构体

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// GitHub 账号配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAccount {
    /// 账号名称（用于标识和切换）
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// 账号邮箱（必填，用于显示和区分）
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub email: String,
    /// GitHub API Token
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_token: String,
}

impl GitHubAccount {
    /// 检查 GitHub 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.email.is_empty() && self.api_token.is_empty()
    }
}

/// GitHub 配置（TOML）
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitHubSettings {
    /// 多个 GitHub 账号列表
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accounts: Vec<GitHubAccount>,
    /// 当前激活的账号名称
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub current: String,
}

impl GitHubSettings {
    /// 检查 GitHub 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty() && self.current.is_empty()
    }

    /// 获取当前激活的账号
    ///
    /// 如果设置了 `current`，返回对应的账号；否则返回第一个账号。
    /// 如果没有账号，返回 `None`。
    pub fn get_current_account(&self) -> Option<&GitHubAccount> {
        if !self.current.is_empty() {
            self.accounts.iter().find(|acc| acc.name == self.current)
        } else {
            self.accounts.first()
        }
    }

    /// 获取当前账号的 API Token
    pub fn get_current_token(&self) -> Option<&str> {
        self.get_current_account().map(|acc| acc.api_token.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_account_is_empty() {
        let empty_account = GitHubAccount {
            name: String::new(),
            email: String::new(),
            api_token: String::new(),
        };
        assert!(empty_account.is_empty());

        let non_empty_account = GitHubAccount {
            name: "test".to_string(),
            email: String::new(),
            api_token: String::new(),
        };
        assert!(!non_empty_account.is_empty());
    }

    #[test]
    fn test_github_account_serialize() {
        let account = GitHubAccount {
            name: "work".to_string(),
            email: "work@example.com".to_string(),
            api_token: "ghp_xxxx".to_string(),
        };

        let toml = toml::to_string(&account).unwrap();
        assert!(toml.contains("name = \"work\""));
        assert!(toml.contains("email = \"work@example.com\""));
    }

    #[test]
    fn test_github_account_deserialize() {
        let toml = r#"
            name = "personal"
            email = "personal@example.com"
            api_token = "ghp_yyyy"
        "#;

        let account: GitHubAccount = toml::from_str(toml).unwrap();
        assert_eq!(account.name, "personal");
        assert_eq!(account.email, "personal@example.com");
        assert_eq!(account.api_token, "ghp_yyyy");
    }

    #[test]
    fn test_github_settings_is_empty() {
        let empty_settings = GitHubSettings::default();
        assert!(empty_settings.is_empty());

        let non_empty_settings = GitHubSettings {
            accounts: vec![GitHubAccount {
                name: "test".to_string(),
                email: "test@example.com".to_string(),
                api_token: "token".to_string(),
            }],
            current: String::new(),
        };
        assert!(!non_empty_settings.is_empty());
    }

    #[test]
    fn test_github_settings_get_current_account_with_current_set() {
        let settings = GitHubSettings {
            accounts: vec![
                GitHubAccount {
                    name: "work".to_string(),
                    email: "work@example.com".to_string(),
                    api_token: "work_token".to_string(),
                },
                GitHubAccount {
                    name: "personal".to_string(),
                    email: "personal@example.com".to_string(),
                    api_token: "personal_token".to_string(),
                },
            ],
            current: "personal".to_string(),
        };

        let current = settings.get_current_account().unwrap();
        assert_eq!(current.name, "personal");
        assert_eq!(current.api_token, "personal_token");
    }

    #[test]
    fn test_github_settings_get_current_account_fallback_to_first() {
        let settings = GitHubSettings {
            accounts: vec![
                GitHubAccount {
                    name: "work".to_string(),
                    email: "work@example.com".to_string(),
                    api_token: "work_token".to_string(),
                },
            ],
            current: String::new(),
        };

        let current = settings.get_current_account().unwrap();
        assert_eq!(current.name, "work");
    }

    #[test]
    fn test_github_settings_get_current_account_returns_none_when_empty() {
        let settings = GitHubSettings::default();
        assert!(settings.get_current_account().is_none());
    }

    #[test]
    fn test_github_settings_get_current_account_returns_none_when_not_found() {
        let settings = GitHubSettings {
            accounts: vec![
                GitHubAccount {
                    name: "work".to_string(),
                    email: "work@example.com".to_string(),
                    api_token: "work_token".to_string(),
                },
            ],
            current: "nonexistent".to_string(),
        };

        assert!(settings.get_current_account().is_none());
    }

    #[test]
    fn test_github_settings_get_current_token() {
        let settings = GitHubSettings {
            accounts: vec![
                GitHubAccount {
                    name: "test".to_string(),
                    email: "test@example.com".to_string(),
                    api_token: "ghp_test_token".to_string(),
                },
            ],
            current: "test".to_string(),
        };

        assert_eq!(settings.get_current_token(), Some("ghp_test_token"));
    }

    #[test]
    fn test_github_settings_serialize_skip_empty() {
        let settings = GitHubSettings {
            accounts: vec![],
            current: String::new(),
        };

        let toml = toml::to_string(&settings).unwrap();
        // 空配置应该生成空字符串（因为 skip_serializing_if）
        assert!(!toml.contains("accounts"));
        assert!(!toml.contains("current"));
    }

    #[test]
    fn test_github_settings_deserialize_with_multiple_accounts() {
        let toml = r#"
            current = "work"

            [[accounts]]
            name = "work"
            email = "work@example.com"
            api_token = "work_token"

            [[accounts]]
            name = "personal"
            email = "personal@example.com"
            api_token = "personal_token"
        "#;

        let settings: GitHubSettings = toml::from_str(toml).unwrap();
        assert_eq!(settings.accounts.len(), 2);
        assert_eq!(settings.current, "work");
        assert_eq!(settings.get_current_account().unwrap().name, "work");
    }
}
