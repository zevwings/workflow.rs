use anyhow::Result;

/// PR 平台接口 trait
/// 定义所有 PR 平台（GitHub、Codeup 等）必须实现的共同方法
pub trait Platform {
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
    fn merge_pull_request(pull_request_id: &str, delete_branch: bool) -> Result<()>;

    /// 获取 PR 信息
    ///
    /// # Arguments
    /// * `pull_request_id_or_branch` - PR ID 或分支名（某些平台支持通过分支名查找）
    ///
    /// # Returns
    /// PR 信息的格式化字符串
    fn get_pull_request_info(pull_request_id_or_branch: &str) -> Result<String>;

    /// 获取 PR URL
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    ///
    /// # Returns
    /// PR URL 字符串
    fn get_pull_request_url(pull_request_id: &str) -> Result<String>;

    /// 获取 PR 标题
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    ///
    /// # Returns
    /// PR 标题字符串
    fn get_pull_request_title(pull_request_id: &str) -> Result<String>;

    /// 获取当前分支的 PR ID
    ///
    /// # Returns
    /// PR ID（如果存在），否则返回 None
    fn get_current_branch_pull_request() -> Result<Option<String>>;

    /// 列出 PR（可选方法，某些平台可能不支持）
    ///
    /// # Arguments
    /// * `state` - PR 状态筛选（如 "open", "closed"）
    /// * `limit` - 返回数量限制
    ///
    /// # Returns
    /// PR 列表的格式化字符串
    fn get_pull_requests(_state: Option<&str>, _limit: Option<u32>) -> Result<String> {
        // 默认实现：返回不支持的错误
        anyhow::bail!("get_pull_requests is not supported by this platform")
    }
}
