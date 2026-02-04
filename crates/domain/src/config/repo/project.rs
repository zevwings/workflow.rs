//! 项目配置类型定义

use crate::config::repo::template::TemplateConfig;
use serde::{Deserialize, Serialize};

/// 项目配置（团队标准）
///
/// 用于解析 `.workflow/config.toml` 文件。
/// 仓库级别的公共配置，提交到 Git。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// 是否使用 scope（当没有 ticket id 时）
    ///
    /// 当 `true` 时，使用 Conventional Commits 格式：`{commit_type}({scope}): {title}`
    /// 当 `false` 时，使用简单格式：`# {title}`
    ///
    /// 默认值为 `false`，表示不使用 scope。
    #[serde(default = "default_use_scope", skip_serializing_if = "is_false")]
    pub use_scope: bool,
    /// 模板配置
    #[serde(default, skip_serializing_if = "TemplateConfig::is_empty")]
    pub template: TemplateConfig,
}

fn default_use_scope() -> bool {
    false
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl ProjectConfig {
    /// 检查项目配置是否为空
    ///
    /// 当 `use_scope` 为默认值（false）且 `template` 为默认值时，认为配置为空。
    pub fn is_empty(&self) -> bool {
        !self.use_scope && self.template == TemplateConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_config_is_empty() {
        let config = ProjectConfig::default();
        assert!(config.is_empty());

        let config = ProjectConfig {
            use_scope: true,
            template: TemplateConfig::default(),
        };
        assert!(!config.is_empty());
    }

    #[test]
    fn test_project_config_serialize_skip_empty() {
        let config = ProjectConfig::default();
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.trim().is_empty());
    }
}
