//! Jira 工作历史记录管理
//!
//! 本模块提供了 PR 创建和合并的工作历史记录管理功能：
//! - 读取工作历史记录（通过 PR ID 查找 Jira ticket）
//! - 根据分支名查找 PR ID
//! - 写入工作历史记录
//! - 更新工作历史记录的合并时间
//! - 删除工作历史记录条目

use crate::{DeleteHistoryResult, JiraError, WorkHistoryEntry};

// ============================================================================
// Repository Trait
// ============================================================================

/// Jira 工作历史记录仓储接口
///
/// 提供 PR 创建和合并的工作历史记录读写功能。
pub trait JiraWorkHistoryRepository: Send + Sync {
    /// 读取工作历史记录（通过 PR ID 查找 Jira ticket）
    ///
    /// 从工作历史记录文件中查找指定 PR ID 对应的 Jira ticket。
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - Pull Request ID（如 `"456"`）
    /// * `repository` - 仓库地址（如 `"git@github.com:owner/repo.git"`）
    ///
    /// # 返回
    ///
    /// 返回 Jira ticket ID（如果找到），否则返回 `None`。
    fn read_work_history(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<Option<String>, JiraError>;

    /// 读取完整的工作历史记录条目
    ///
    /// 从工作历史记录文件中读取指定 PR ID 的完整记录。
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - Pull Request ID（如 `"456"`）
    /// * `repository` - 仓库地址
    ///
    /// # 返回
    ///
    /// 返回 `WorkHistoryEntry` 结构体（如果找到），否则返回 `None`。
    fn read_work_history_entry(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<Option<WorkHistoryEntry>, JiraError>;

    /// 根据分支名从工作历史记录中查找 PR ID
    ///
    /// # 参数
    ///
    /// * `branch_name` - 分支名称（如 `"feature/PROJ-123-add-feature"`）
    /// * `repository` - 仓库地址
    ///
    /// # 返回
    ///
    /// 返回 PR ID（如果找到），否则返回 `None`。
    fn find_pr_id_by_branch(
        &self,
        branch_name: &str,
        repository: &str,
    ) -> Result<Option<String>, JiraError>;

    /// 写入工作历史记录
    ///
    /// 将 PR 创建信息写入工作历史记录文件。
    /// 如果记录已存在，则更新；如果不存在，则创建新记录。
    ///
    /// # 参数
    ///
    /// * `jira_ticket` - Jira ticket ID（如 `"PROJ-123"`）
    /// * `pull_request_id` - Pull Request ID（如 `"456"`）
    /// * `pull_request_url` - Pull Request URL（可选）
    /// * `repository` - 仓库地址
    /// * `branch` - 分支名称（可选）
    fn write_work_history(
        &self,
        jira_ticket: &str,
        pull_request_id: &str,
        pull_request_url: Option<&str>,
        repository: &str,
        branch: Option<&str>,
    ) -> Result<(), JiraError>;

    /// 更新工作历史记录的合并时间
    ///
    /// 更新指定 PR ID 的工作历史记录，设置 `merged_at` 为当前时间。
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - Pull Request ID（如 `"456"`）
    /// * `repository` - 仓库地址
    fn update_work_history_merged(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<(), JiraError>;

    /// 删除工作历史记录中的 PR ID 条目
    ///
    /// 从工作历史记录文件中删除指定 PR ID 的条目。
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - Pull Request ID（如 `"157"`）
    /// * `repository` - 仓库地址
    ///
    /// # 返回
    ///
    /// 返回 `DeleteHistoryResult`，包含删除操作的消息。
    fn delete_work_history_entry(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<DeleteHistoryResult, JiraError>;

    /// 根据 JIRA ticket 查找关联的 PR 列表
    ///
    /// 从所有仓库的工作历史记录中查找与指定 JIRA ticket 关联的 PR。
    ///
    /// # 参数
    ///
    /// * `jira_ticket` - Jira ticket ID（如 `"PROJ-123"`）
    ///
    /// # 返回
    ///
    /// 返回 `Vec<WorkHistoryEntry>`，包含所有关联的 PR 信息。
    fn find_prs_by_jira_ticket(
        &self,
        jira_ticket: &str,
    ) -> Result<Vec<WorkHistoryEntry>, JiraError>;

    /// 根据 JIRA ticket 查找关联的分支列表
    ///
    /// # 参数
    ///
    /// * `jira_ticket` - Jira ticket ID（如 `"PROJ-123"`）
    ///
    /// # 返回
    ///
    /// 返回 `Vec<String>`，包含所有关联的分支名称。
    fn find_branches_by_jira_ticket(&self, jira_ticket: &str) -> Result<Vec<String>, JiraError>;
}
