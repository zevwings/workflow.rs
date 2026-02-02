//! Status 服务实体定义

use domain::ProjectStatusConfig as DomainProjectStatusConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Jira 用户条目（TOML）
///
/// 用于存储单个用户的 Jira 信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraUserEntry {
    /// 用户邮箱（必需，用于查找）
    pub email: String,
    /// 账户 ID
    pub account_id: String,
    /// 显示名称
    pub display_name: String,
}

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
    /// 用户列表
    #[serde(default)]
    pub users: Vec<JiraUserEntry>,

    /// 项目状态配置映射
    /// 使用 `[status.PROJECT_KEY]` 格式存储每个项目的状态配置
    #[serde(default, rename = "status")]
    pub status: HashMap<String, DomainProjectStatusConfig>,
}
