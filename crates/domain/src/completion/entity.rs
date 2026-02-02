//! Shell Completion 实体

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Completion 配置结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionConfigResult {
    /// Shell 类型
    pub shell: String,
    /// 是否已存在（如果为 true，表示配置已存在，未进行修改）
    pub already_exists: bool,
    /// 是否成功添加（如果为 true，表示新添加了配置）
    pub added: bool,
    /// 配置文件路径（如果适用）
    pub config_file: Option<PathBuf>,
}

/// Completion 文件删除结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRemovalResult {
    /// 删除的文件数量
    pub removed_count: usize,
    /// 删除的文件列表
    pub removed_files: Vec<PathBuf>,
    /// 失败的文件列表（文件路径和错误信息）
    pub failed_files: Vec<(PathBuf, String)>,
}

/// Completion 管理工具
///
/// 提供 Shell Completion 的配置和管理功能。
/// 当前功能通过服务层实现，此实体为未来业务逻辑封装预留。
pub struct Completion;
