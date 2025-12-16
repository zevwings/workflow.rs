//! 目录遍历工具
//!
//! 提供目录遍历相关的工具函数，统一使用 `walkdir` 进行目录遍历。

use color_eyre::{eyre::WrapErr, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 列出目录中的所有子目录
///
/// 递归遍历指定目录，返回所有子目录的路径。
///
/// # 参数
///
/// * `path` - 根目录路径
///
/// # 返回
///
/// 返回所有子目录的路径向量。
///
/// # 错误
///
/// 如果目录遍历失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::directory::list_dirs;
/// use std::path::PathBuf;
///
/// # fn main() -> color_eyre::Result<()> {
/// let dirs = list_dirs(PathBuf::from("/tmp").as_path())?;
/// for dir in dirs {
///     println!("{:?}", dir);
/// }
/// # Ok(())
/// # }
/// ```
pub fn list_dirs(path: &Path) -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry.wrap_err_with(|| format!("Failed to read directory entry: {:?}", path))?;
        if entry.file_type().is_dir() {
            dirs.push(entry.path().to_path_buf());
        }
    }
    Ok(dirs)
}

/// 列出目录中的所有文件
///
/// 递归遍历指定目录，返回所有文件的路径。
///
/// # 参数
///
/// * `path` - 根目录路径
///
/// # 返回
///
/// 返回所有文件的路径向量。
///
/// # 错误
///
/// 如果目录遍历失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::directory::list_files;
/// use std::path::PathBuf;
///
/// # fn main() -> color_eyre::Result<()> {
/// let files = list_files(PathBuf::from("/tmp").as_path())?;
/// for file in files {
///     println!("{:?}", file);
/// }
/// # Ok(())
/// # }
/// ```
pub fn list_files(path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry.wrap_err_with(|| format!("Failed to read directory entry: {:?}", path))?;
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
    Ok(files)
}

/// 查找匹配模式的文件
///
/// 递归遍历指定目录，返回所有匹配指定模式的文件路径。
/// 模式可以是文件名的一部分或完整文件名。
///
/// # 参数
///
/// * `path` - 根目录路径
/// * `pattern` - 文件名模式（支持部分匹配）
///
/// # 返回
///
/// 返回所有匹配模式的文件路径向量。
///
/// # 错误
///
/// 如果目录遍历失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::directory::find_files;
/// use std::path::PathBuf;
///
/// # fn main() -> color_eyre::Result<()> {
/// let files = find_files(PathBuf::from("/tmp").as_path(), ".toml")?;
/// for file in files {
///     println!("{:?}", file);
/// }
/// # Ok(())
/// # }
/// ```
pub fn find_files(path: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry.wrap_err_with(|| format!("Failed to read directory entry: {:?}", path))?;
        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_string_lossy();
            if file_name.contains(pattern) {
                files.push(entry.path().to_path_buf());
            }
        }
    }
    Ok(files)
}

/// 列出目录中的直接子目录（非递归）
///
/// 只列出指定目录的直接子目录，不递归遍历。
///
/// # 参数
///
/// * `path` - 目录路径
///
/// # 返回
///
/// 返回直接子目录的路径向量。
///
/// # 错误
///
/// 如果目录读取失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::directory::list_direct_dirs;
/// use std::path::PathBuf;
///
/// # fn main() -> color_eyre::Result<()> {
/// let dirs = list_direct_dirs(PathBuf::from("/tmp").as_path())?;
/// for dir in dirs {
///     println!("{:?}", dir);
/// }
/// # Ok(())
/// # }
/// ```
pub fn list_direct_dirs(path: &Path) -> Result<Vec<PathBuf>> {
    use crate::base::util::path::read_dir_safe;
    let entries = read_dir_safe(path)?;
    Ok(entries
        .into_iter()
        .filter(|p| p.is_dir())
        .collect())
}

/// 列出目录中的直接文件（非递归）
///
/// 只列出指定目录的直接文件，不递归遍历。
///
/// # 参数
///
/// * `path` - 目录路径
///
/// # 返回
///
/// 返回直接文件的路径向量。
///
/// # 错误
///
/// 如果目录读取失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::directory::list_direct_files;
/// use std::path::PathBuf;
///
/// # fn main() -> color_eyre::Result<()> {
/// let files = list_direct_files(PathBuf::from("/tmp").as_path())?;
/// for file in files {
///     println!("{:?}", file);
/// }
/// # Ok(())
/// # }
/// ```
pub fn list_direct_files(path: &Path) -> Result<Vec<PathBuf>> {
    use crate::base::util::path::read_dir_safe;
    let entries = read_dir_safe(path)?;
    Ok(entries
        .into_iter()
        .filter(|p| p.is_file())
        .collect())
}
