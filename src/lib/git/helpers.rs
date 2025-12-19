//! git2 工具函数
//!
//! 提供 git2 库的辅助函数，用于打开仓库和处理常见操作。

use color_eyre::{eyre::WrapErr, Result};
use git2::Repository;

/// 打开当前目录的 Git 仓库
///
/// # 返回
///
/// 返回打开的 `Repository` 对象。
///
/// # 错误
///
/// 如果当前目录不是 Git 仓库或打开失败，返回相应的错误信息。
pub fn open_repo() -> Result<Repository> {
    Repository::open(".").wrap_err("Failed to open repository")
}

/// 打开指定路径的 Git 仓库
///
/// # 参数
///
/// * `path` - 仓库路径
///
/// # 返回
///
/// 返回打开的 `Repository` 对象。
///
/// # 错误
///
/// 如果指定路径不是 Git 仓库或打开失败，返回相应的错误信息。
#[allow(dead_code)]
pub fn open_repo_at(path: &str) -> Result<Repository> {
    Repository::open(path).wrap_err_with(|| format!("Failed to open repository at: {}", path))
}
