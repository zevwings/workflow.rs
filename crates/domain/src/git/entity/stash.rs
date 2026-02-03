//! Stash 实体类型

use chrono::{DateTime, Local};

/// Stash 条目信息
#[derive(Debug, Clone)]
pub struct StashEntry {
    /// stash@{n} 中的 n
    pub index: usize,
    /// 创建时的分支
    pub branch: String,
    /// stash 消息
    pub message: String,
    /// commit hash
    pub commit_hash: String,
    /// 创建时间
    pub timestamp: Option<DateTime<Local>>,
}

/// Stash 应用结果
#[derive(Debug, Clone)]
pub struct StashApplyResult {
    /// 是否成功应用
    pub applied: bool,
    /// 是否有冲突
    pub has_conflicts: bool,
    /// 消息
    pub message: Option<String>,
    /// 警告消息列表
    pub warnings: Vec<String>,
    /// 统计信息（可选）
    pub stat: Option<StashStat>,
}

/// Stash 统计信息
#[derive(Debug, Clone)]
pub struct StashStat {
    /// 变更的文件数
    pub files_changed: usize,
    /// 插入的行数
    pub insertions: usize,
    /// 删除的行数
    pub deletions: usize,
}

/// Stash 恢复结果
#[derive(Debug, Clone)]
pub struct StashPopResult {
    /// 是否成功恢复
    pub restored: bool,
    /// 消息
    pub message: Option<String>,
    /// 警告消息列表
    pub warnings: Vec<String>,
}
