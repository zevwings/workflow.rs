//! Git 操作辅助函数
//!
//! 提供 git2 相关的工具函数，用于简化 Git 仓库操作。

use color_eyre::{eyre::WrapErr, Result};
use git2::Repository;

/// 打开当前目录的 Git 仓库
///
/// 从当前工作目录开始向上查找 `.git` 目录，打开 Git 仓库。
///
/// # 返回
///
/// 返回打开的 `Repository` 对象。
///
/// # 错误
///
/// 如果不在 Git 仓库中或打开失败，返回相应的错误信息。
pub fn open_repo() -> Result<Repository> {
    Repository::open(".")
        .wrap_err("Failed to open Git repository. Make sure you're in a Git repository.")
}

/// 打开指定路径的 Git 仓库
///
/// 从指定路径开始向上查找 `.git` 目录，打开 Git 仓库。
///
/// # 参数
///
/// * `path` - 仓库路径（可以是仓库根目录或子目录）
///
/// # 返回
///
/// 返回打开的 `Repository` 对象。
///
/// # 错误
///
/// 如果不在 Git 仓库中或打开失败，返回相应的错误信息。
pub fn open_repo_at(path: impl AsRef<std::path::Path>) -> Result<Repository> {
    Repository::open(path.as_ref())
        .wrap_err_with(|| format!("Failed to open Git repository at: {:?}", path.as_ref()))
}
