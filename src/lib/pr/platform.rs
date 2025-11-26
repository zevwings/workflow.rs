use crate::git::{GitRepo, RepoType};
use crate::pr::codeup::Codeup;
use crate::pr::github::GitHub;
use anyhow::Result;

/// PR 变更类型定义
///
/// 用于生成 PR body 中的变更类型复选框和交互式选择
pub const TYPES_OF_CHANGES: &[&str] = &[
    "Bug fix (non-breaking change which fixes an issue)",
    "New feature (non-breaking change which adds functionality)",
    "Refactoring (non-breaking change which does not change functionality)",
];

/// PR 状态信息
#[derive(Debug, Clone)]
pub struct PullRequestStatus {
    /// PR 状态（如 "open", "closed", "merged"）
    pub state: String,
    /// 是否已合并
    pub merged: bool,
    /// 合并时间（如果已合并）
    pub merged_at: Option<String>,
}

/// PR 平台接口 trait
/// 定义所有 PR 平台（GitHub、Codeup 等）必须实现的共同方法
pub trait PlatformProvider {
    /// 创建 Pull Request
    ///
    /// # Arguments
    /// * `title` - PR 标题
    /// * `body` - PR 描述
    /// * `source_branch` - 源分支名
    /// * `target_branch` - 目标分支名（可选，默认由各平台决定）
    ///
    /// # Returns
    /// PR URL 字符串
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: Option<&str>,
    ) -> Result<String>;

    /// 合并 Pull Request
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    /// * `delete_branch` - 是否删除源分支
    fn merge_pull_request(&self, pull_request_id: &str, delete_branch: bool) -> Result<()>;

    /// 获取 PR 信息
    ///
    /// # Arguments
    /// * `pull_request_id_or_branch` - PR ID 或分支名（某些平台支持通过分支名查找）
    ///
    /// # Returns
    /// PR 信息的格式化字符串
    fn get_pull_request_info(&self, pull_request_id_or_branch: &str) -> Result<String>;

    /// 获取 PR URL
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    ///
    /// # Returns
    /// PR URL 字符串
    fn get_pull_request_url(&self, pull_request_id: &str) -> Result<String>;

    /// 获取 PR 标题
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    ///
    /// # Returns
    /// PR 标题字符串
    fn get_pull_request_title(&self, pull_request_id: &str) -> Result<String>;

    /// 获取当前分支的 PR ID
    ///
    /// # Returns
    /// PR ID（如果存在），否则返回 None
    fn get_current_branch_pull_request(&self) -> Result<Option<String>>;

    /// 列出 PR（可选方法，某些平台可能不支持）
    ///
    /// # Arguments
    /// * `state` - PR 状态筛选（如 "open", "closed"）
    /// * `limit` - 返回数量限制
    ///
    /// # Returns
    /// PR 列表的格式化字符串
    fn get_pull_requests(&self, _state: Option<&str>, _limit: Option<u32>) -> Result<String> {
        // 默认实现：返回不支持的错误
        anyhow::bail!("get_pull_requests is not supported by this platform")
    }

    /// 获取 PR 状态
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    ///
    /// # Returns
    /// PR 状态信息，包含是否已合并等信息
    fn get_pull_request_status(&self, pull_request_id: &str) -> Result<PullRequestStatus>;

    /// 关闭 Pull Request
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    fn close_pull_request(&self, pull_request_id: &str) -> Result<()>;

    /// 获取 PR 的 diff 内容
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    ///
    /// # Returns
    /// PR 的 diff 内容（字符串格式）
    fn get_pull_request_diff(&self, _pull_request_id: &str) -> Result<String> {
        // 默认实现：返回不支持的错误
        anyhow::bail!("get_pull_request_diff is not supported by this platform")
    }
}

/// 创建平台提供者实例
///
/// 根据当前仓库类型自动检测并创建对应的平台提供者。
///
/// # 返回
///
/// 返回 `Box<dyn PlatformProvider>` trait 对象，可以用于调用平台无关的 PR 操作。
///
/// # 错误
///
/// 如果仓库类型未知或不支持，返回错误。
///
/// # 示例
///
/// ```rust,no_run
/// use crate::pr::create_provider;
///
/// let provider = create_provider()?;
/// let pr_url = provider.create_pull_request(
///     "Title",
///     "Body",
///     "feature-branch",
///     None,
/// )?;
/// ```
pub fn create_provider() -> Result<Box<dyn PlatformProvider>> {
    match GitRepo::detect_repo_type()? {
        RepoType::GitHub => Ok(Box::new(GitHub)),
        RepoType::Codeup => Ok(Box::new(Codeup)),
        RepoType::Unknown => {
            anyhow::bail!("Unsupported repository type. Only GitHub and Codeup are supported.")
        }
    }
}
