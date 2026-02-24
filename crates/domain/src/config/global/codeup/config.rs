//! Codeup 配置相关结构体

use serde::{Deserialize, Serialize};

/// Codeup 配置（TOML）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeupSettings {
    /// Codeup 项目 ID
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub project_id: String,
    /// Codeup CSRF Token
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub csrf_token: String,
    /// Codeup Cookie
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub cookie: String,
}

impl CodeupSettings {
    /// 检查 Codeup 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.project_id.is_empty() && self.csrf_token.is_empty() && self.cookie.is_empty()
    }

    /// 检查 Codeup 配置是否完整（用于 API 调用）
    pub fn is_complete(&self) -> bool {
        !self.project_id.is_empty() && !self.csrf_token.is_empty() && !self.cookie.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codeup_settings_is_empty() {
        let empty_settings = CodeupSettings::default();
        assert!(empty_settings.is_empty());

        let non_empty_settings = CodeupSettings {
            project_id: "12345".to_string(),
            csrf_token: String::new(),
            cookie: String::new(),
        };
        assert!(!non_empty_settings.is_empty());
    }

    #[test]
    fn test_codeup_settings_is_complete() {
        let incomplete_settings = CodeupSettings {
            project_id: "12345".to_string(),
            csrf_token: String::new(),
            cookie: String::new(),
        };
        assert!(!incomplete_settings.is_complete());

        let complete_settings = CodeupSettings {
            project_id: "12345".to_string(),
            csrf_token: "csrf_token_value".to_string(),
            cookie: "cookie_value".to_string(),
        };
        assert!(complete_settings.is_complete());
    }

    #[test]
    fn test_codeup_settings_serialize() {
        let settings = CodeupSettings {
            project_id: "12345".to_string(),
            csrf_token: "csrf_token_value".to_string(),
            cookie: "cookie_value".to_string(),
        };

        let toml = toml::to_string(&settings).unwrap();
        assert!(toml.contains("project_id = \"12345\""));
        assert!(toml.contains("csrf_token = \"csrf_token_value\""));
        assert!(toml.contains("cookie = \"cookie_value\""));
    }

    #[test]
    fn test_codeup_settings_deserialize() {
        let toml = r#"
            project_id = "12345"
            csrf_token = "csrf_token_value"
            cookie = "cookie_value"
        "#;

        let settings: CodeupSettings = toml::from_str(toml).unwrap();
        assert_eq!(settings.project_id, "12345");
        assert_eq!(settings.csrf_token, "csrf_token_value");
        assert_eq!(settings.cookie, "cookie_value");
    }

    #[test]
    fn test_codeup_settings_serialize_skip_empty() {
        let settings = CodeupSettings::default();
        let toml = toml::to_string(&settings).unwrap();
        // 空配置应该生成空字符串（因为 skip_serializing_if）
        assert!(!toml.contains("project_id"));
        assert!(!toml.contains("csrf_token"));
        assert!(!toml.contains("cookie"));
    }
}
