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
