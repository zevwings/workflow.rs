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

/// 提交中单个文件的变更信息
#[derive(Debug, Clone)]
pub struct CommitFileChange {
    /// 当前路径（新文件路径或重命名后的路径）
    pub path: String,
    /// 变更类型
    pub change_type: CommitChangeType,
    /// 原路径（仅重命名/复制时有值）
    pub old_path: Option<String>,
    /// 该文件在本次提交中新增的行数（仅文本 diff 时有值）
    pub additions: Option<u32>,
    /// 该文件在本次提交中删除的行数（仅文本 diff 时有值）
    pub deletions: Option<u32>,
}

/// 提交内文件变更类型（与 git diff 的 delta 对应）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitChangeType {
    /// 新增文件
    Added,
    /// 修改
    Modified,
    /// 删除
    Deleted,
    /// 重命名
    Renamed,
    /// 复制
    Copied,
    /// 类型变更（如 submodule -> 普通文件）
    TypeChanged,
}
