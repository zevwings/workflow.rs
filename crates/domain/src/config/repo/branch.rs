//! 分支配置相关结构体

use serde::{Deserialize, Serialize};

/// 分支配置（个人偏好）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BranchConfig {
    /// 分支前缀（个人偏好）
    ///
    /// 空字符串表示未配置前缀。
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub prefix: String,
    /// 忽略的分支列表（个人偏好）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore: Vec<String>,
}

impl BranchConfig {
    /// 检查分支配置是否为空
    ///
    /// 当 `prefix` 为空字符串且 `ignore` 为空时，认为配置为空。
    pub fn is_empty(&self) -> bool {
        self.prefix.is_empty() && self.ignore.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_config_default() {
        let config = BranchConfig::default();
        assert!(config.prefix.is_empty());
        assert!(config.ignore.is_empty());
    }

    #[test]
    fn test_branch_config_is_empty() {
        let empty_config = BranchConfig::default();
        assert!(empty_config.is_empty());

        let config_with_prefix = BranchConfig {
            prefix: "zw".to_string(),
            ignore: vec![],
        };
        assert!(!config_with_prefix.is_empty());

        let config_with_ignore = BranchConfig {
            prefix: String::new(),
            ignore: vec!["main".to_string()],
        };
        assert!(!config_with_ignore.is_empty());
    }

    #[test]
    fn test_branch_config_serialize() {
        let config = BranchConfig {
            prefix: "dev".to_string(),
            ignore: vec!["main".to_string(), "develop".to_string()],
        };

        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("prefix = \"dev\""));
        assert!(toml.contains("ignore = ["));
    }

    #[test]
    fn test_branch_config_deserialize() {
        let toml = r#"
            prefix = "feature"
            ignore = ["main", "master", "develop"]
        "#;

        let config: BranchConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.prefix, "feature");
        assert_eq!(config.ignore.len(), 3);
        assert!(config.ignore.contains(&"main".to_string()));
        assert!(config.ignore.contains(&"master".to_string()));
        assert!(config.ignore.contains(&"develop".to_string()));
    }

    #[test]
    fn test_branch_config_deserialize_partial() {
        let toml = r#"
            prefix = "test"
        "#;

        let config: BranchConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.prefix, "test");
        assert!(config.ignore.is_empty());
    }

    #[test]
    fn test_branch_config_serialize_skip_empty() {
        let config = BranchConfig {
            prefix: "zw".to_string(),
            ignore: vec![],
        };

        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("prefix"));
        assert!(!toml.contains("ignore"));
    }

    #[test]
    fn test_branch_config_roundtrip() {
        let original = BranchConfig {
            prefix: "roundtrip".to_string(),
            ignore: vec!["branch1".to_string(), "branch2".to_string()],
        };

        let toml = toml::to_string(&original).unwrap();
        let deserialized: BranchConfig = toml::from_str(&toml).unwrap();

        assert_eq!(original.prefix, deserialized.prefix);
        assert_eq!(original.ignore, deserialized.ignore);
    }

    #[test]
    fn test_branch_config_clone() {
        let config = BranchConfig {
            prefix: "clone".to_string(),
            ignore: vec!["ignored".to_string()],
        };

        let cloned = config.clone();
        assert_eq!(config.prefix, cloned.prefix);
        assert_eq!(config.ignore, cloned.ignore);
    }

    #[test]
    fn test_branch_config_empty_ignore_list() {
        let toml = r#"
            prefix = "empty"
            ignore = []
        "#;

        let config: BranchConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.prefix, "empty");
        assert!(config.ignore.is_empty());
    }
}
