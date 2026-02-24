//! Codeup 仓储接口
//!
//! 定义与 Codeup REST API 交互的底层接口。

use crate::{
    codeup::{entity::CodeupUser, error::CodeupError},
    pr::entity::PullRequestInfo,
};

/// Codeup 仓储接口
///
/// 提供与 Codeup REST API 交互的底层接口，封装了 Pull Request 和用户信息的操作。
pub trait CodeupRepository: Send + Sync {
    /// 创建 Pull Request
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<String, CodeupError>;

    /// 获取 Pull Request 信息
    fn get_pull_request(&self, pr_id: &str) -> Result<PullRequestInfo, CodeupError>;

    /// 合并 Pull Request
    fn merge_pull_request(&self, pr_id: &str, force: bool) -> Result<(), CodeupError>;

    /// 获取用户信息
    fn get_user_info(&self) -> Result<CodeupUser, CodeupError>;

    /// 关闭 Pull Request
    fn close_pull_request(&self, pr_id: &str) -> Result<(), CodeupError>;

    /// 列出 Pull Requests
    fn list_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PullRequestInfo>, CodeupError>;

    /// 更新 Pull Request 的标题和/或描述
    fn update_pull_request(
        &self,
        pr_id: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<(), CodeupError>;

    /// 添加评论到 Pull Request
    fn add_comment(&self, pr_id: &str, comment: &str) -> Result<(), CodeupError>;

    /// 批准 Pull Request
    fn approve_pull_request(&self, pr_id: &str) -> Result<(), CodeupError>;

    /// 获取 Pull Request 的 diff 内容
    fn get_pr_diff(&self, pr_id: &str) -> Result<String, CodeupError>;

    /// 获取 PR 信息（格式化字符串）
    fn get_pull_request_info(&self, pr_id: &str) -> Result<String, CodeupError>;

    /// 获取 PR URL
    fn get_pull_request_url(&self, pr_id: &str) -> Result<String, CodeupError>;

    /// 获取 PR 标题
    fn get_pull_request_title(&self, pr_id: &str) -> Result<String, CodeupError>;

    /// 获取 PR body 内容
    fn get_pull_request_body(&self, pr_id: &str) -> Result<Option<String>, CodeupError>;

    /// 获取 PR 状态
    fn get_pull_request_status(
        &self,
        pr_id: &str,
    ) -> Result<(String, bool, Option<String>), CodeupError>;

    /// 更新 PR 的 base 分支
    fn update_pr_base(&self, pr_id: &str, new_base: &str) -> Result<(), CodeupError>;

    /// 获取当前分支的 PR ID
    fn get_current_branch_pull_request(
        &self,
        current_branch: &str,
    ) -> Result<Option<String>, CodeupError>;
}
