//! 仓库配置类型定义

use crate::config::repo::{BranchConfig, MCPConfig, ProjectConfig, UserConfig};
use serde::{Deserialize, Serialize};

/// 仓库配置（统一接口）
///
/// 组合项目配置（ProjectConfig）和用户配置（UserConfig），
/// 提供统一的访问接口。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoConfig {
    /// 项目配置（团队标准，提交到 Git）
    #[serde(default, skip_serializing_if = "ProjectConfig::is_empty")]
    pub project: ProjectConfig,
    /// 用户配置（个人偏好，不提交到 Git）
    #[serde(default, skip_serializing_if = "UserConfig::is_empty")]
    pub user: UserConfig,
    /// MCP 配置（项目级配置，`.cursor/mcp.json`）
    #[serde(default, skip_serializing_if = "MCPConfig::is_empty")]
    pub mcp: MCPConfig,
}

impl RepoConfig {
    /// 获取分支配置（优先使用用户配置，回退到项目配置）
    ///
    /// 如果用户配置中有分支配置（非空），则返回用户配置；
    /// 否则返回 `None`（项目配置中不包含分支配置）。
    pub fn get_branch_config(&self) -> Option<&BranchConfig> {
        if !self.user.branch.is_empty() {
            Some(&self.user.branch)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_config_get_branch_config_prefers_user() {
        let config = RepoConfig {
            project: ProjectConfig::default(),
            user: UserConfig {
                branch: BranchConfig {
                    prefix: "zw".to_string(),
                    ignore: vec![],
                },
            },
            mcp: MCPConfig::default(),
        };

        let branch = config.get_branch_config().expect("branch config should exist");
        assert_eq!(branch.prefix, "zw");
    }

    #[test]
    fn test_repo_config_get_branch_config_none_when_empty() {
        let config = RepoConfig::default();
        assert!(config.get_branch_config().is_none());
    }
}
