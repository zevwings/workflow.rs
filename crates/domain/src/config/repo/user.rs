//! 用户配置类型定义

use serde::{Deserialize, Serialize};

use crate::config::repo::branch::BranchConfig;

/// 用户配置（个人偏好）
///
/// 用于解析 `.workflow/user.toml` 文件。
/// 仓库级别的个人偏好配置，不提交到 Git。
///
/// 格式：
/// ```toml
/// [branch]
/// prefix = "zw"
/// ignore = ["branch1", "branch2"]
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserConfig {
    /// 分支配置（个人偏好）
    #[serde(default, skip_serializing_if = "BranchConfig::is_empty")]
    pub branch: BranchConfig,
}

impl UserConfig {
    /// 检查用户配置是否为空
    ///
    /// 当 `branch` 为空配置时，认为配置为空。
    pub fn is_empty(&self) -> bool {
        self.branch.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_config_is_empty() {
        let config = UserConfig::default();
        assert!(config.is_empty());

        let config = UserConfig {
            branch: BranchConfig {
                prefix: "dev".to_string(),
                ignore: vec![],
            },
        };
        assert!(!config.is_empty());
    }

    #[test]
    fn test_user_config_serialize_skip_empty() {
        let config = UserConfig::default();
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.trim().is_empty());
    }
}
