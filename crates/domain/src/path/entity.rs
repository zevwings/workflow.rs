//! LLM 实体类型
//!
//! 包含 LLM 服务返回的实体类型定义

use std::path::PathBuf;

use serde::Deserialize;

/// PR 创建内容，包含分支名、PR 标题、描述、scope 和详细总结
///
/// 由 LLM 生成的分支名、PR 标题、描述、scope 和详细总结，用于创建 Pull Request。
#[derive(Debug, Clone, Deserialize)]
pub struct Dir {
    /// 是否可以使用 iCloud 基础目录
    pub is_icloud_available: bool,
    /// iCloud 基础目录
    pub icloud_base_dir: PathBuf,
    /// 本地基础目录
    pub local_base_dir: PathBuf,
}
