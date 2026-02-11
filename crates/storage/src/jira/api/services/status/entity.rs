//! Status 服务实体定义

use std::collections::HashMap;

use domain::ProjectStatusConfig;
use serde::{Deserialize, Serialize};

/// 合并后的 Jira 配置
///
/// 将 `jira-users.toml` 和 `jira-status.toml` 合并为 `jira.toml`。
///
/// TOML 格式示例：
/// ```toml
/// [[users]]
/// email = "user@example.com"
/// account_id = "628d9616269a9a0068f27e0c"
/// display_name = "User Name"
///
/// [status.WEW]
/// created-pr = "In Progress"
/// merged-pr = "In Review"
///
/// [status.NA]
/// created-pr = "In Progress"
/// merged-pr = "In Review"
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JiraConfig {
    /// 项目状态配置映射
    /// 使用 `[status.PROJECT_KEY]` 格式存储每个项目的状态配置
    #[serde(default, rename = "status", skip_serializing_if = "HashMap::is_empty")]
    pub status: HashMap<String, ProjectStatusConfig>,
}
