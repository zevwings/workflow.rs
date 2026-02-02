//! 提交相关实体

/// 提交信息
///
/// 包含完整的提交元数据，支持 git2 提供的丰富信息
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// 完整 SHA（40 字符）
    pub sha: String,
    /// 完整提交消息
    pub message: String,
    /// 提交消息摘要（第一行）
    pub summary: String,
    /// 作者名称
    pub author_name: String,
    /// 作者邮箱
    pub author_email: String,
    /// 作者时间（Unix 时间戳）
    pub author_time: i64,
    /// 提交者名称
    pub committer_name: String,
    /// 提交者邮箱
    pub committer_email: String,
    /// 提交时间（Unix 时间戳）
    pub committer_time: i64,
    /// 父提交 SHA 列表
    pub parents: Vec<String>,
}
