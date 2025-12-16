//! 路径操作工具
//!
//! 提供路径相关的工具函数，包括：
//! - 路径验证
//! - 目录创建和检查
//! - 路径操作辅助函数

use color_eyre::{eyre::WrapErr, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// 确保目录存在
///
/// 如果目录不存在，则创建它（包括所有父目录）。
///
/// # 参数
///
/// * `path` - 目录路径
///
/// # 返回
///
/// 如果目录已存在或创建成功，返回 `Ok(())`。
///
/// # 错误
///
/// 如果目录创建失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::path::ensure_dir_exists;
/// use std::path::PathBuf;
///
/// # fn main() -> color_eyre::Result<()> {
/// ensure_dir_exists(PathBuf::from("/tmp/my_dir").as_path())?;
/// # Ok(())
/// # }
/// ```
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .wrap_err_with(|| format!("Failed to create directory: {:?}", path))?;
    }
    Ok(())
}

/// 确保父目录存在
///
/// 如果指定路径的父目录不存在，则创建它（包括所有父目录）。
///
/// # 参数
///
/// * `path` - 文件或目录路径
///
/// # 返回
///
/// 如果父目录已存在或创建成功，返回 `Ok(())`。
///
/// # 错误
///
/// 如果父目录创建失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::path::ensure_parent_dir_exists;
/// use std::path::PathBuf;
///
/// # fn main() -> color_eyre::Result<()> {
/// let file_path = PathBuf::from("/tmp/my_dir/file.txt");
/// ensure_parent_dir_exists(&file_path)?;
/// # Ok(())
/// # }
/// ```
pub fn ensure_parent_dir_exists(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir_exists(parent)?;
    }
    Ok(())
}

/// 安全地读取目录内容
///
/// 读取目录中的所有条目，忽略无法读取的条目，返回成功读取的路径列表。
///
/// # 参数
///
/// * `path` - 目录路径
///
/// # 返回
///
/// 返回目录中所有条目的路径向量。
///
/// # 错误
///
/// 如果目录读取失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::path::read_dir_safe;
/// use std::path::PathBuf;
///
/// # fn main() -> color_eyre::Result<()> {
/// let entries = read_dir_safe(PathBuf::from("/tmp").as_path())?;
/// for entry in entries {
///     println!("{:?}", entry);
/// }
/// # Ok(())
/// # }
/// ```
pub fn read_dir_safe(path: &Path) -> Result<Vec<PathBuf>> {
    let entries =
        fs::read_dir(path).wrap_err_with(|| format!("Failed to read directory: {:?}", path))?;
    let mut paths = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => paths.push(entry.path()),
            Err(_) => {
                // 忽略无法读取的条目
                continue;
            }
        }
    }
    Ok(paths)
}

/// 检查路径是否存在
///
/// 检查指定路径是否存在（文件或目录）。
///
/// # 参数
///
/// * `path` - 路径
///
/// # 返回
///
/// 如果路径存在返回 `true`，否则返回 `false`。
pub fn path_exists(path: &Path) -> bool {
    path.exists()
}

/// 检查路径是否为文件
///
/// 检查指定路径是否为文件。
///
/// # 参数
///
/// * `path` - 路径
///
/// # 返回
///
/// 如果路径是文件返回 `true`，否则返回 `false`。
pub fn is_file(path: &Path) -> bool {
    path.is_file()
}

/// 检查路径是否为目录
///
/// 检查指定路径是否为目录。
///
/// # 参数
///
/// * `path` - 路径
///
/// # 返回
///
/// 如果路径是目录返回 `true`，否则返回 `false`。
pub fn is_dir(path: &Path) -> bool {
    path.is_dir()
}
