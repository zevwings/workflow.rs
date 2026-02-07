//! PR（Pull Request）服务接口
//!
//! 提供完整的 Pull Request 生命周期管理功能，包括创建、更新、合并、关闭等操作。

use crate::errors::ServiceError;
use crate::pr::entity::PullRequestInfo;

/// PR 服务接口
///
/// 提供与 Git 托管平台（如 GitHub、GitLab、Bitbucket）交互的 Pull Request 管理功能。
///
/// # 功能特性
///
/// - **生命周期管理**：创建、更新、合并、关闭 PR
/// - **智能生成**：可选基于 LLM 生成标题和描述
/// - **状态查询**：查询 PR 状态、列表、详情
/// - **协作功能**：评论、批准 PR
/// - **Diff 查看**：获取 PR 的完整 diff 内容
///
/// # 线程安全
///
/// 实现须满足 [`Send`] + [`Sync`]，以便在多线程或异步上下文中共享。
///
/// # 示例
///
/// ```ignore
/// use domain::PullRequestService;
///
/// fn example(service: &dyn PullRequestService) -> Result<(), Box<dyn std::error::Error>> {
///     // 创建 PR（自动生成标题和描述）
///     let pr_id = service.create_pull_request(
///         Some("JIRA-123"),
///         None,  // 自动生成标题
///         None,  // 自动生成描述
///         None,  // 使用默认目标分支
///     )?;
///     println!("Created PR: {}", pr_id);
///
///     // 获取 PR 状态
///     let status = service.get_pr_status(Some(&pr_id))?;
///     println!("PR state: {}", status.state);
///
///     // 添加评论
///     service.add_comment(&pr_id, "LGTM!")?;
///
///     // 合并 PR
///     service.merge_pull_request(&pr_id, false)?;
///
///     Ok(())
/// }
/// ```
pub trait PullRequestService: Send + Sync {
    /// 创建 Pull Request
    ///
    /// 创建一个新的 PR，支持自动生成标题和描述。如果未提供标题，
    /// 将使用 LLM 基于变更内容自动生成。
    ///
    /// # 参数
    ///
    /// * `jira_id` - JIRA issue ID（可选）。如果提供，会自动关联到 JIRA ticket
    /// * `title` - PR 标题（可选）。如果为 `None`，使用 LLM 自动生成
    /// * `description` - PR 描述内容（可选）。如果为 `None`，使用 LLM 自动生成
    /// * `target_branch` - 目标分支名（可选）。如果为 `None`，使用仓库默认分支（main/master）
    ///
    /// # 返回
    ///
    /// 返回创建的 PR ID（字符串格式，如 "123" 或 "owner/repo#123"）
    ///
    /// # 错误
    ///
    /// * [`ServiceError::Git`] - Git 操作失败（无法推送、分支不存在等）
    /// * [`ServiceError::GitHub`] - GitHub API 调用失败（权限不足、网络错误等）
    /// * [`ServiceError::Other`] - LLM 生成标题/描述失败（仅当未提供标题时）或其他错误
    fn create_pull_request(
        &self,
        jira_id: Option<&str>,
        title: Option<&str>,
        description: Option<&str>,
        target_branch: Option<&str>,
    ) -> Result<String, ServiceError>;

    /// 合并 Pull Request
    ///
    /// 将指定的 PR 合并到目标分支。
    ///
    /// # 参数
    ///
    /// * `pr_id` - PR ID
    /// * `force` - 是否强制合并（忽略 CI 检查失败、审批要求等）
    ///
    /// # 错误
    ///
    /// * [`ServiceError::GitHub`] - 合并失败（冲突、权限不足、CI 未通过等）
    /// * [`ServiceError::Other`] - 其他错误
    ///
    /// # 注意
    ///
    /// - `force = true` 可能绕过仓库保护规则，需谨慎使用
    /// - 合并后源分支不会自动删除，需手动清理
    fn merge_pull_request(&self, pr_id: &str, force: bool) -> Result<(), ServiceError>;

    /// 获取 PR 状态
    ///
    /// 查询指定 PR 或当前分支的 PR 状态。
    ///
    /// # 参数
    ///
    /// * `pr_id_or_branch` - PR ID 或分支名。如果为 `None`，查询当前分支的 PR
    ///
    /// # 返回
    ///
    /// 返回 [`PrStatus`]，包含 PR 的 ID、标题、状态和合并状态
    ///
    /// # 错误
    ///
    /// * [`ServiceError::GitHub`] - PR 不存在或 API 调用失败
    /// * [`ServiceError::Git`] - 无法确定当前分支
    /// * [`ServiceError::NotFound`] - 当前分支没有关联的 PR
    fn get_pr_status(&self, pr_id_or_branch: Option<&str>) -> Result<PrStatus, ServiceError>;

    /// 关闭 Pull Request
    ///
    /// 关闭指定的 PR，但不合并。适用于废弃的 PR 或需要重新提交的场景。
    ///
    /// # 参数
    ///
    /// * `pr_id` - PR ID
    ///
    /// # 错误
    ///
    /// * [`ServiceError::GitHub`] - PR 不存在或已关闭
    /// * [`ServiceError::Other`] - 其他错误
    ///
    /// # 注意
    ///
    /// - 关闭的 PR 可以重新打开（如果平台支持）
    /// - 源分支不会自动删除
    fn close_pull_request(&self, pr_id: &str) -> Result<(), ServiceError>;

    /// 列出 Pull Requests
    ///
    /// 查询仓库中的 PR 列表，支持按状态筛选和限制数量。
    ///
    /// # 参数
    ///
    /// * `state` - PR 状态筛选，可选值：
    ///   - `Some("open")` - 仅开放的 PR
    ///   - `Some("closed")` - 仅关闭的 PR
    ///   - `Some("merged")` - 仅已合并的 PR
    ///   - `Some("all")` - 所有 PR
    ///   - `None` - 默认为 "open"
    /// * `limit` - 返回数量限制。如果为 `None`，使用平台默认值（通常 30-100）
    ///
    /// # 返回
    ///
    /// 返回 [`PrStatus`] 列表，按更新时间倒序排列
    ///
    /// # 错误
    ///
    /// * [`ServiceError::GitHub`] - API 调用失败
    /// * [`ServiceError::Other`] - 其他错误
    fn list_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PrStatus>, ServiceError>;

    /// 更新 Pull Request 的标题和/或描述
    ///
    /// 修改已存在的 PR 的标题或描述。至少需要提供标题或描述中的一个。
    ///
    /// # 参数
    ///
    /// * `pr_id` - PR ID
    /// * `title` - 新的标题（可选）。如果为 `None`，保持原标题不变
    /// * `body` - 新的描述（可选）。如果为 `None`，保持原描述不变
    ///
    /// # 错误
    ///
    /// * [`ServiceError::GitHub`] - PR 不存在或 API 调用失败
    /// * [`ServiceError::Other`] - 其他错误
    ///
    /// # 注意
    ///
    /// - 如果 `title` 和 `body` 均为 `None`，操作将成功但不会有任何变更
    fn update_pull_request(
        &self,
        pr_id: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<(), ServiceError>;

    /// 添加评论到 Pull Request
    ///
    /// 在指定 PR 的讨论区添加一条评论。
    ///
    /// # 参数
    ///
    /// * `pr_id` - PR ID
    /// * `comment` - 评论内容（支持 Markdown 格式）
    ///
    /// # 错误
    ///
    /// * [`ServiceError::GitHub`] - PR 不存在或权限不足
    /// * [`ServiceError::Other`] - 其他错误
    fn add_comment(&self, pr_id: &str, comment: &str) -> Result<(), ServiceError>;

    /// 批准 Pull Request
    ///
    /// 对指定 PR 进行批准（Approve）操作，表示代码审查通过。
    ///
    /// # 参数
    ///
    /// * `pr_id` - PR ID
    ///
    /// # 错误
    ///
    /// * [`ServiceError::GitHub`] - PR 不存在或权限不足（不能批准自己的 PR）
    /// * [`ServiceError::Other`] - 其他错误
    ///
    /// # 注意
    ///
    /// - 在某些平台，不能批准自己创建的 PR
    /// - 批准不代表自动合并，仍需调用 `merge_pull_request`
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

/// PR 状态信息
///
/// 表示 Pull Request 的当前状态，包括 ID、标题、状态和合并状态。
///
/// # 字段
///
/// * `id` - PR 的唯一标识符（如 "123"）
/// * `title` - PR 的标题
/// * `state` - PR 的状态（"open", "closed", "merged"）
/// * `merged` - 是否已合并。`true` 表示 PR 已合并，`false` 表示未合并
///
/// # 注意
///
/// - `state = "closed"` 且 `merged = true` 表示 PR 已合并
/// - `state = "closed"` 且 `merged = false` 表示 PR 已关闭但未合并
/// - `state = "open"` 且 `merged = true` 是不可能的状态
#[derive(Debug, Clone)]
pub struct PrStatus {
    /// PR ID
    pub id: String,
    /// PR 标题
    pub title: String,
    /// PR 状态（open/closed/merged）
    pub state: String,
    /// 是否已合并
    pub merged: bool,
}
