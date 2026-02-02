//! 用户配置类型定义

use crate::config::repo::branch::BranchConfig;
use serde::{Deserialize, Serialize};

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
