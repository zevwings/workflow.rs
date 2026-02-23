//! 目录管理工具
//!
//! 提供目录遍历、创建和路径检查的工具函数。

use std::{fs, path::Path};

use walkdir::WalkDir;

use crate::util::fs::FileError;

// ============================================================================
// 目录遍历函数
// ============================================================================

/// 递归列出所有子目录。
///
/// # 参数
///
/// * `path` - 根目录路径
///
/// # 返回
///
/// 返回所有子目录的路径列表（包括根目录）。
pub fn list_dirs(path: &Path) -> Result<Vec<std::path::PathBuf>, FileError> {
    let mut dirs = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry.map_err(|e| {
            FileError::Other(format!("Failed to read directory entry {:?}: {}", path, e))
        })?;
        if entry.file_type().is_dir() {
            dirs.push(entry.path().to_path_buf());
        }
    }
    Ok(dirs)
}

/// 递归列出所有文件。
///
/// # 参数
///
/// * `path` - 根目录路径
///
/// # 返回
///
/// 返回所有文件的路径列表。
pub fn list_files(path: &Path) -> Result<Vec<std::path::PathBuf>, FileError> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry.map_err(|e| {
            FileError::Other(format!("Failed to read directory entry {:?}: {}", path, e))
        })?;
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
    Ok(files)
}

/// 递归查找匹配模式的文件（文件名包含给定模式）。
///
/// # 参数
///
/// * `path` - 根目录路径
/// * `pattern` - 文件名匹配模式
///
/// # 返回
///
/// 返回匹配文件的路径列表。
pub fn find_files(path: &Path, pattern: &str) -> Result<Vec<std::path::PathBuf>, FileError> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry.map_err(|e| {
            FileError::Other(format!("Failed to read directory entry: {:?}: {}", path, e))
        })?;
        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_string_lossy();
            if file_name.contains(pattern) {
                files.push(entry.path().to_path_buf());
            }
        }
    }
    Ok(files)
}

/// 非递归列出直接子目录。
///
/// # 参数
///
/// * `path` - 目录路径
///
/// # 返回
///
/// 返回直接子目录的路径列表。
pub fn list_direct_dirs(path: &Path) -> Result<Vec<std::path::PathBuf>, FileError> {
    let entries = fs::read_dir(path).map_err(FileError::Io)?;
    let mut dirs = Vec::new();
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            dirs.push(entry_path);
        }
    }
    Ok(dirs)
}

/// 非递归列出直接文件。
///
/// # 参数
///
/// * `path` - 目录路径
///
/// # 返回
///
/// 返回直接文件的路径列表。
pub fn list_direct_files(path: &Path) -> Result<Vec<std::path::PathBuf>, FileError> {
    let entries = fs::read_dir(path).map_err(FileError::Io)?;
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_file() {
            files.push(entry_path);
        }
    }
    Ok(files)
}

// ============================================================================
// 目录创建函数
// ============================================================================

/// 确保目录存在，如果不存在则创建。
///
/// # 参数
///
/// * `path` - 目录路径
///
/// # 返回
///
/// 成功返回 `Ok(())`，失败返回错误。
///
/// # 示例
///
/// ```rust
/// use std::path::Path;
/// use toolkit::directory;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let test_dir = std::env::temp_dir().join("test_dir");
/// directory::ensure_exists(&test_dir)?;
/// # Ok(())
/// # }
/// ```
pub fn ensure_exists(path: &Path) -> Result<(), FileError> {
    fs::create_dir_all(path).map_err(FileError::Io)
}

/// 确保文件的父目录存在，如果不存在则创建。
///
/// # 参数
///
/// * `file_path` - 文件路径，将创建其父目录
///
/// # 返回
///
/// 成功返回 `Ok(())`，失败返回错误。
///
/// # 示例
///
/// ```rust
/// use std::path::Path;
/// use toolkit::directory;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let file_path = std::env::temp_dir().join("some/nested/file.txt");
/// directory::ensure_parent_exists(&file_path)?;
/// # Ok(())
/// # }
/// ```
pub fn ensure_parent_exists(file_path: &Path) -> Result<(), FileError> {
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).map_err(FileError::Io)?;
    }
    Ok(())
}

/// 安全读取目录条目，忽略读取失败的条目。
///
/// # 参数
///
/// * `path` - 目录路径
///
/// # 返回
///
/// 返回所有条目的路径列表。
pub fn read_dir_safe(path: &Path) -> Result<Vec<std::path::PathBuf>, FileError> {
    let entries = fs::read_dir(path)
        .map_err(|e| FileError::Other(format!("Failed to read directory: {:?}: {}", path, e)))?;
    let mut paths = Vec::new();
    for entry in entries.flatten() {
        paths.push(entry.path());
    }
    Ok(paths)
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use tempfile::tempdir;

    use super::*;

    // ==================== 目录遍历测试 ====================

    #[test]
    fn test_list_dirs_recursive() -> Result<(), FileError> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // 创建目录结构
        fs::create_dir_all(root.join("dir1/subdir1"))?;
        fs::create_dir_all(root.join("dir2"))?;
        fs::create_dir_all(root.join("dir1/subdir2"))?;

        let dirs = list_dirs(root)?;

        // 应该包含根目录和所有子目录
        assert!(dirs.contains(&root.to_path_buf()));
        assert!(dirs.iter().any(|d| d.ends_with("dir1")));
        assert!(dirs.iter().any(|d| d.ends_with("subdir1")));
        assert!(dirs.iter().any(|d| d.ends_with("subdir2")));
        assert!(dirs.iter().any(|d| d.ends_with("dir2")));

        Ok(())
    }

    #[test]
    fn test_list_files_recursive() -> Result<(), FileError> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // 创建文件和目录结构
        fs::create_dir_all(root.join("subdir"))?;
        fs::write(root.join("file1.txt"), "content1")?;
        fs::write(root.join("file2.txt"), "content2")?;
        fs::write(root.join("subdir/file3.txt"), "content3")?;

        let files = list_files(root)?;

        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.ends_with("file1.txt")));
        assert!(files.iter().any(|f| f.ends_with("file2.txt")));
        assert!(files.iter().any(|f| f.ends_with("file3.txt")));

        Ok(())
    }

    #[test]
    fn test_find_files() -> Result<(), FileError> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // 创建文件
        fs::write(root.join("test_file.txt"), "content")?;
        fs::write(root.join("other_file.txt"), "content")?;
        fs::write(root.join("test_config.toml"), "content")?;
        fs::create_dir_all(root.join("subdir"))?;
        fs::write(root.join("subdir/test_file.rs"), "content")?;

        let files = find_files(root, "test")?;

        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.ends_with("test_file.txt")));
        assert!(files.iter().any(|f| f.ends_with("test_config.toml")));
        assert!(files.iter().any(|f| f.ends_with("test_file.rs")));

        Ok(())
    }

    #[test]
    fn test_list_direct_dirs() -> Result<(), FileError> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // 创建目录结构
        fs::create_dir_all(root.join("dir1/subdir"))?;
        fs::create_dir_all(root.join("dir2"))?;

        let dirs = list_direct_dirs(root)?;

        // 应该只包含直接子目录，不包含子目录的子目录
        assert_eq!(dirs.len(), 2);
        assert!(dirs.iter().any(|d| d.ends_with("dir1")));
        assert!(dirs.iter().any(|d| d.ends_with("dir2")));
        assert!(!dirs.iter().any(|d| d.ends_with("subdir")));

        Ok(())
    }

    #[test]
    fn test_list_direct_files() -> Result<(), FileError> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // 创建文件和目录结构
        fs::create_dir_all(root.join("subdir"))?;
        fs::write(root.join("file1.txt"), "content1")?;
        fs::write(root.join("file2.txt"), "content2")?;
        fs::write(root.join("subdir/file3.txt"), "content3")?;

        let files = list_direct_files(root)?;

        // 应该只包含直接文件，不包含子目录中的文件
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.ends_with("file1.txt")));
        assert!(files.iter().any(|f| f.ends_with("file2.txt")));
        assert!(!files.iter().any(|f| f.ends_with("file3.txt")));

        Ok(())
    }

    // ==================== 目录创建测试 ====================

    #[test]
    fn test_ensure_exists() -> Result<(), FileError> {
        let temp_dir = tempdir()?;
        let new_dir = temp_dir.path().join("new_dir");

        assert!(!new_dir.exists());

        ensure_exists(&new_dir)?;
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());

        Ok(())
    }

    #[test]
    fn test_ensure_exists_nested() -> Result<(), FileError> {
        let temp_dir = tempdir()?;
        let nested_dir = temp_dir.path().join("level1/level2/level3");

        ensure_exists(&nested_dir)?;

        assert!(nested_dir.exists());
        assert!(nested_dir.is_dir());

        Ok(())
    }

    #[test]
    fn test_ensure_parent_exists() -> Result<(), FileError> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("subdir/nested/file.txt");

        ensure_parent_exists(&file_path)?;

        let parent = file_path.parent().expect("File path should have a parent directory");
        assert!(parent.exists());
        assert!(parent.is_dir());

        Ok(())
    }

    #[test]
    fn test_ensure_parent_exists_for_root_path() -> Result<(), FileError> {
        let result = ensure_parent_exists(Path::new("/"));

        // 根路径没有父目录，应该成功（不执行任何操作）
        assert!(result.is_ok());

        Ok(())
    }

    // ==================== 安全读取测试 ====================

    #[test]
    fn test_read_dir_safe() -> Result<(), FileError> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // 创建文件和目录
        fs::write(root.join("file1.txt"), "content1")?;
        fs::write(root.join("file2.txt"), "content2")?;
        fs::create_dir_all(root.join("subdir"))?;

        let entries = read_dir_safe(root)?;

        // 应该包含所有条目（文件和目录）
        assert!(entries.len() >= 3);
        assert!(entries.iter().any(|e| e.ends_with("file1.txt")));
        assert!(entries.iter().any(|e| e.ends_with("file2.txt")));
        assert!(entries.iter().any(|e| e.ends_with("subdir")));

        Ok(())
    }

    // ==================== 边界情况测试 ====================

    #[test]
    fn test_list_dirs_empty_directory() -> Result<(), FileError> {
        let temp_dir = tempdir()?;
        let dirs = list_dirs(temp_dir.path())?;

        // 应该至少包含根目录本身
        assert!(!dirs.is_empty());
        assert!(dirs.contains(&temp_dir.path().to_path_buf()));

        Ok(())
    }

    #[test]
    fn test_list_files_empty_directory() -> Result<(), FileError> {
        let temp_dir = tempdir()?;
        let files = list_files(temp_dir.path())?;

        assert_eq!(files.len(), 0);

        Ok(())
    }

    #[test]
    fn test_find_files_no_match() -> Result<(), FileError> {
        let temp_dir = tempdir()?;
        fs::write(temp_dir.path().join("file1.txt"), "content")?;
        fs::write(temp_dir.path().join("file2.txt"), "content")?;

        let files = find_files(temp_dir.path(), "nonexistent")?;

        assert_eq!(files.len(), 0);

        Ok(())
    }
}
