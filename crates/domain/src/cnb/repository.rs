//! CNB 仓储接口

use crate::cnb::entity::CNBUser;
use crate::cnb::error::CNBError;
use crate::pr::entity::PullRequestInfo;

/// CNB 仓储接口
///
/// 提供 CNB API 操作的接口定义。
pub trait CNBRepository: Send + Sync {
    /// 创建 Pull Request
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<String, CNBError>; // 返回 PR ID

    /// 获取 Pull Request 信息
    fn get_pull_request(&self, pr_id: &str) -> Result<PullRequestInfo, CNBError>;

    /// 合并 Pull Request
    fn merge_pull_request(&self, pr_id: &str, force: bool) -> Result<(), CNBError>;

    /// 获取用户信息
    fn get_user_info(&self) -> Result<CNBUser, CNBError>;

    /// 关闭 Pull Request
    ///
    /// # 参数
    /// * `pr_id` - PR ID
    fn close_pull_request(&self, pr_id: &str) -> Result<(), CNBError>;

    /// 列出 Pull Requests
    ///
    /// # 参数
    /// * `state` - PR 状态筛选（如 "open", "closed", "merged"）
    /// * `limit` - 返回数量限制
    ///
    /// # 返回
    /// Pull Request 信息列表
    fn list_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PullRequestInfo>, CNBError>;

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
    ) -> Result<(), CNBError>;

    /// 添加评论到 Pull Request
    ///
    /// # 参数
    /// * `pr_id` - PR ID
    /// * `comment` - 评论内容
    fn add_comment(&self, pr_id: &str, comment: &str) -> Result<(), CNBError>;

    /// 批准 Pull Request
    ///
    /// # 参数
    /// * `pr_id` - PR ID
    fn approve_pull_request(&self, pr_id: &str) -> Result<(), CNBError>;

    /// 获取 Pull Request 的 diff 内容
    ///
    /// # 参数
    /// * `pr_id` - PR ID
    ///
    /// # 返回
    /// PR 的 diff 内容（字符串格式）
    fn get_pr_diff(&self, pr_id: &str) -> Result<String, CNBError>;

    /// 获取 PR 信息（格式化字符串）
    ///
    /// # 参数
    /// * `pr_id` - PR ID
    ///
    /// # 返回
    /// 格式化的 PR 信息字符串
    fn get_pull_request_info(&self, pr_id: &str) -> Result<String, CNBError>;

    /// 获取 PR URL
    ///
    /// # 参数
    /// * `pr_id` - PR ID
    ///
    /// # 返回
    /// PR 的 URL
    fn get_pull_request_url(&self, pr_id: &str) -> Result<String, CNBError>;

    /// 获取 PR 标题
    ///
    /// # 参数
    /// * `pr_id` - PR ID
    ///
    /// # 返回
    /// PR 的标题
    fn get_pull_request_title(&self, pr_id: &str) -> Result<String, CNBError>;

    /// 获取 PR body 内容
    ///
    /// # 参数
    /// * `pr_id` - PR ID
    ///
    /// # 返回
    /// PR 的 body 内容（可能为空）
    fn get_pull_request_body(&self, pr_id: &str) -> Result<Option<String>, CNBError>;

    /// 获取 PR 状态
    ///
    /// # 参数
    /// * `pr_id` - PR ID
    ///
    /// # 返回
    /// 元组：(状态, 是否已合并, 合并时间)
    fn get_pull_request_status(
        &self,
        pr_id: &str,
    ) -> Result<(String, bool, Option<String>), CNBError>;

    /// 更新 PR 的 base 分支
    ///
    /// # 参数
    /// * `pr_id` - PR ID
    /// * `new_base` - 新的 base 分支名
    fn update_pr_base(&self, pr_id: &str, new_base: &str) -> Result<(), CNBError>;

    /// 获取当前分支的 PR ID
    ///
    /// # 返回
    /// 当前分支关联的 PR ID，如果不存在则返回 `None`
    fn get_current_branch_pull_request(
        &self,
        current_branch: &str,
    ) -> Result<Option<String>, CNBError>;
}
