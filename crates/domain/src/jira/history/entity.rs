use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// 工作历史记录条目
///
/// 记录 PR 的创建和合并信息，包括 Jira ticket、PR URL、时间戳等。
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkHistoryEntry {
    /// Jira ticket ID（如 `"PROJ-123"`）
    pub jira_ticket: String,
    /// Pull Request URL（可选）
    pub pull_request_url: Option<String>,
    /// PR 创建时间（ISO 8601 格式，可选）
    pub created_at: Option<String>,
    /// PR 合并时间（ISO 8601 格式，可选）
    pub merged_at: Option<String>,
    /// 仓库地址（可选）
    pub repository: Option<String>,
    /// 分支名称（可选）
    pub branch: Option<String>,
}

/// 删除工作历史记录结果
#[derive(Debug, Clone)]
pub struct DeleteHistoryResult {
    /// 删除的消息列表
    pub messages: Vec<String>,
    /// 警告消息列表
    pub warnings: Vec<String>,
}
