//! 全局配置类型定义

use crate::config::global::github::config::GitHubSettings;
use crate::config::global::jira::config::JiraSettings;
use crate::config::global::llm::config::LLMSettings;
use crate::config::global::log::config::LogSettings;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 全局配置
/// 从 workflow.toml 配置文件读取配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Jira 配置
    #[serde(default, skip_serializing_if = "JiraSettings::is_empty")]
    pub jira: JiraSettings,
    /// GitHub 配置
    #[serde(default, skip_serializing_if = "GitHubSettings::is_empty")]
    pub github: GitHubSettings,
    /// 日志配置
    #[serde(default, skip_serializing_if = "LogSettings::is_empty")]
    pub log: LogSettings,
    /// LLM 配置
    #[serde(default, skip_serializing_if = "LLMSettings::is_empty")]
    pub llm: LLMSettings,
    /// 别名配置（TOML section: `[aliases]`）
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub aliases: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_config_serialize_skip_empty() {
        let config = GlobalConfig::default();
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.trim().is_empty());
    }

    #[test]
    fn test_global_config_roundtrip_with_aliases_and_jira() {
        let mut aliases = HashMap::new();
        aliases.insert("co".to_string(), "checkout".to_string());

        let config = GlobalConfig {
            jira: JiraSettings {
                email: "user@example.com".to_string(),
                api_token: "token".to_string(),
                service_address: "https://jira.example.com".to_string(),
            },
            github: GitHubSettings::default(),
            log: LogSettings::default(),
            llm: LLMSettings::default(),
            aliases,
        };

        let toml = toml::to_string(&config).unwrap();
        let deserialized: GlobalConfig = toml::from_str(&toml).unwrap();

        assert_eq!(deserialized.jira.email, "user@example.com");
        assert_eq!(
            deserialized.aliases.get("co").expect("alias should exist"),
            "checkout"
        );
    }
}
