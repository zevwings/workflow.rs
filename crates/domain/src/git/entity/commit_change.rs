//! 单次提交中的文件变更实体

/// 提交中单个文件的变更信息
#[derive(Debug, Clone)]
pub struct CommitFileChange {
    /// 当前路径（新文件路径或重命名后的路径）
    pub path: String,
    /// 变更类型
    pub change_type: CommitChangeType,
    /// 原路径（仅重命名/复制时有值）
    pub old_path: Option<String>,
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
