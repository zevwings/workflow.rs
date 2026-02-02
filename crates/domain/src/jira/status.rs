//! Jira 状态配置类型

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// 状态配置结果
#[derive(Debug, Clone)]
pub struct StatusConfigResult {
    /// 项目名称
    pub project: String,
    /// PR 创建时的目标状态
    pub created_pull_request_status: String,
    /// PR 合并时的目标状态
    pub merged_pull_request_status: String,
}

/// 项目状态配置
///
/// 存储单个项目的状态配置，包括 PR 创建和合并时的目标状态。
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatusConfig {
    /// PR 创建时的目标状态（JSON 字段名：`created-pr`）
    #[serde(rename = "created-pr")]
    pub created_pull_request_status: Option<String>,
    /// PR 合并时的目标状态（JSON 字段名：`merged-pr`）
    #[serde(rename = "merged-pr")]
    pub merged_pull_request_status: Option<String>,
}

/// Jira 状态配置
///
/// 包含项目名称和对应的状态配置。
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatusConfig {
    /// 项目名称（如 `"PROJ"`）
    pub project: String,
    /// PR 创建时的目标状态
    pub created_pull_request_status: Option<String>,
    /// PR 合并时的目标状态
    pub merged_pull_request_status: Option<String>,
}
