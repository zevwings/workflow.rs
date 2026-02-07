//! PR 服务接口

use crate::errors::ServiceError;
use crate::pr::entity::PullRequestInfo;

/// PR 服务接口
pub trait PullRequestService: Send + Sync {
    /// 创建 Pull Request
    ///
    /// # 参数
    /// * `jira_id` - JIRA ID（可选）
    /// * `title` - PR 标题（可选，不提供时使用 LLM 生成）
    /// * `description` - PR 描述（可选）
    /// * `target_branch` - 目标分支（可选，不提供时使用仓库默认分支）
    fn create_pull_request(
        &self,
        jira_id: Option<&str>,
        title: Option<&str>,
        description: Option<&str>,
        target_branch: Option<&str>,
    ) -> Result<String, ServiceError>; // 返回 PR ID

    /// 合并 Pull Request
    fn merge_pull_request(&self, pr_id: &str, force: bool) -> Result<(), ServiceError>;

    /// 获取 PR 状态
    fn get_pr_status(&self, pr_id_or_branch: Option<&str>) -> Result<PrStatus, ServiceError>;

    /// 关闭 Pull Request
    fn close_pull_request(&self, pr_id: &str) -> Result<(), ServiceError>;

    /// 列出 Pull Requests
    ///
    /// # 参数
    /// * `state` - PR 状态筛选（如 "open", "closed", "merged"）
    /// * `limit` - 返回数量限制
    fn list_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PrStatus>, ServiceError>;

    /// 更新 Pull Request 的标题和/或描述
    ///
    /// # 参数
    /// * `pr_id` - PR ID
    /// * `title` - 新的标题（可选）
    /// * `body` - 新的描述（可选）
    fn update_pull_request(
        &self,
        pr_id: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<(), ServiceError>;

    /// 添加评论到 Pull Request
    ///
    /// # 参数
    /// * `pr_id` - PR ID
    /// * `comment` - 评论内容
    fn add_comment(&self, pr_id: &str, comment: &str) -> Result<(), ServiceError>;

    /// 批准 Pull Request
    ///
    /// # 参数
    /// * `pr_id` - PR ID
    fn approve_pull_request(&self, pr_id: &str) -> Result<(), ServiceError>;

    /// 获取 Pull Request 的 diff 内容
    ///
    /// # 参数
    /// * `pr_id` - PR ID
    ///
    /// # 返回
    /// PR 的 diff 内容（字符串格式）
    fn get_pr_diff(&self, pr_id: &str) -> Result<String, ServiceError>;

    /// 获取 Pull Request 详细信息
    ///
    /// # 参数
    /// * `pr_id` - PR ID
    ///
    /// # 返回
    /// Pull Request 的完整信息
    fn get_pull_request(&self, pr_id: &str) -> Result<PullRequestInfo, ServiceError>;

    /// 获取当前分支的 PR ID
    ///
    /// # 参数
    /// * `current_branch` - 当前分支名
    ///
    /// # 返回
    /// 当前分支关联的 PR ID，如果不存在则返回 `None`
    fn get_current_branch_pull_request(
        &self,
        current_branch: &str,
    ) -> Result<Option<String>, ServiceError>;
}

// ==================== 类型定义 ====================

/// PR 状态
#[derive(Debug, Clone)]
pub struct PrStatus {
    pub id: String,
    pub title: String,
    pub state: String,
    pub merged: bool,
}
