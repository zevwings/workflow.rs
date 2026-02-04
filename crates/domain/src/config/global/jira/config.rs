//! Jira 配置相关结构体

use serde::{Deserialize, Serialize};

/// Jira 配置（TOML）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JiraSettings {
    /// Jira 用户邮箱（用于 API 认证）
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub email: String,
    /// Jira API Token
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_token: String,
    /// Jira 服务地址
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub service_address: String,
}

impl JiraSettings {
    /// 检查 JIRA 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.email.is_empty() && self.api_token.is_empty() && self.service_address.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jira_settings_default() {
        let settings = JiraSettings::default();
        assert!(settings.email.is_empty());
        assert!(settings.api_token.is_empty());
        assert!(settings.service_address.is_empty());
    }

    #[test]
    fn test_jira_settings_is_empty() {
        let empty_settings = JiraSettings::default();
        assert!(empty_settings.is_empty());

        let non_empty_settings = JiraSettings {
            email: "user@example.com".to_string(),
            api_token: String::new(),
            service_address: String::new(),
        };
        assert!(!non_empty_settings.is_empty());
    }

    #[test]
    fn test_jira_settings_serialize() {
        let settings = JiraSettings {
            email: "user@example.com".to_string(),
            api_token: "jira_api_token".to_string(),
            service_address: "https://jira.example.com".to_string(),
        };

        let toml = toml::to_string(&settings).unwrap();
        assert!(toml.contains("email = \"user@example.com\""));
        assert!(toml.contains("api_token = \"jira_api_token\""));
        assert!(toml.contains("service_address = \"https://jira.example.com\""));
    }

    #[test]
    fn test_jira_settings_deserialize() {
        let toml = r#"
            email = "test@example.com"
            api_token = "secret_token"
            service_address = "https://test.atlassian.net"
        "#;

        let settings: JiraSettings = toml::from_str(toml).unwrap();
        assert_eq!(settings.email, "test@example.com");
        assert_eq!(settings.api_token, "secret_token");
        assert_eq!(settings.service_address, "https://test.atlassian.net");
    }

    #[test]
    fn test_jira_settings_deserialize_partial() {
        let toml = r#"
            email = "partial@example.com"
        "#;

        let settings: JiraSettings = toml::from_str(toml).unwrap();
        assert_eq!(settings.email, "partial@example.com");
        assert!(settings.api_token.is_empty());
        assert!(settings.service_address.is_empty());
    }

    #[test]
    fn test_jira_settings_serialize_skip_empty() {
        let settings = JiraSettings {
            email: "user@example.com".to_string(),
            api_token: String::new(),
            service_address: String::new(),
        };

        let toml = toml::to_string(&settings).unwrap();
        assert!(toml.contains("email"));
        assert!(!toml.contains("api_token"));
        assert!(!toml.contains("service_address"));
    }

    #[test]
    fn test_jira_settings_roundtrip() {
        let original = JiraSettings {
            email: "roundtrip@example.com".to_string(),
            api_token: "token123".to_string(),
            service_address: "https://jira.test.com".to_string(),
        };

        let toml = toml::to_string(&original).unwrap();
        let deserialized: JiraSettings = toml::from_str(&toml).unwrap();

        assert_eq!(original.email, deserialized.email);
        assert_eq!(original.api_token, deserialized.api_token);
        assert_eq!(original.service_address, deserialized.service_address);
    }

    #[test]
    fn test_jira_settings_clone() {
        let settings = JiraSettings {
            email: "clone@example.com".to_string(),
            api_token: "clone_token".to_string(),
            service_address: "https://clone.atlassian.net".to_string(),
        };

        let cloned = settings.clone();
        assert_eq!(settings.email, cloned.email);
        assert_eq!(settings.api_token, cloned.api_token);
        assert_eq!(settings.service_address, cloned.service_address);
    }
}
