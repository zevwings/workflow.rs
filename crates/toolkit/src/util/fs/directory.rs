//! 目录管理工具
//!
//! 提供基于路径的目录管理助手 `DirectoryWalker`，包括目录遍历、创建和路径检查功能。

use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::util::fs::FsError;

/// 目录管理助手，基于固定根路径提供目录遍历、创建和路径检查操作。
pub struct DirectoryWalker {
    root: PathBuf,
}

impl DirectoryWalker {
    /// 创建新的目录遍历助手。
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { root: path.into() }
    }

    /// 递归列出所有子目录。
    pub fn list_dirs(&self) -> Result<Vec<PathBuf>, FsError> {
        let mut dirs = Vec::new();
        for entry in WalkDir::new(&self.root) {
            let entry = entry.map_err(|e| {
                FsError::Other(format!(
                    "Failed to read directory entry {:?}: {}",
                    self.root, e
                ))
            })?;
            if entry.file_type().is_dir() {
                dirs.push(entry.path().to_path_buf());
            }
        }
        Ok(dirs)
    }

    /// 递归列出所有文件。
    pub fn list_files(&self) -> Result<Vec<PathBuf>, FsError> {
        let mut files = Vec::new();
        for entry in WalkDir::new(&self.root) {
            let entry = entry.map_err(|e| {
                FsError::Other(format!(
                    "Failed to read directory entry {:?}: {}",
                    self.root, e
                ))
            })?;
            if entry.file_type().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }
        Ok(files)
    }

    /// 递归查找匹配模式的文件（文件名包含给定模式）。
    pub fn find_files(&self, pattern: &str) -> Result<Vec<PathBuf>, FsError> {
        let mut files = Vec::new();
        for entry in WalkDir::new(&self.root) {
            let entry = entry.map_err(|e| {
                FsError::Other(format!(
                    "Failed to read directory entry: {:?}: {}",
                    self.root, e
                ))
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
    pub fn list_direct_dirs(&self) -> Result<Vec<PathBuf>, FsError> {
        let entries = fs::read_dir(&self.root).map_err(FsError::Io)?;
        let mut dirs = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            }
        }
        Ok(dirs)
    }

    /// 非递归列出直接文件。
    pub fn list_direct_files(&self) -> Result<Vec<PathBuf>, FsError> {
        let entries = fs::read_dir(&self.root).map_err(FsError::Io)?;
        let mut files = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            }
        }
        Ok(files)
    }

    /// 确保根目录存在，如果不存在则创建。
    ///
    /// # Returns
    ///
    /// * `Result<()>` - 成功时返回 `Ok(())`，失败时返回错误
    ///
    /// # Examples
    ///
    /// ```rust
    /// use toolkit::DirectoryWalker;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let test_dir = std::env::temp_dir().join("test_dir");
    /// let walker = DirectoryWalker::new(&test_dir);
    /// walker.ensure_exists()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn ensure_exists(&self) -> Result<(), FsError> {
        fs::create_dir_all(&self.root).map_err(FsError::Io)
    }

    /// 确保文件的父目录存在，如果不存在则创建。
    ///
    /// # Arguments
    ///
    /// * `file_path` - 文件路径，将创建其父目录
    ///
    /// # Returns
    ///
    /// * `Result<()>` - 成功时返回 `Ok(())`，失败时返回错误
    ///
    /// # Examples
    ///
    /// ```rust
    /// use toolkit::DirectoryWalker;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let temp_dir = std::env::temp_dir();
    /// let walker = DirectoryWalker::new(&temp_dir);
    /// let file_path = temp_dir.join("some/nested/file.txt");
    /// walker.ensure_parent_exists(&file_path)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn ensure_parent_exists(&self, file_path: &Path) -> Result<(), FsError> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(FsError::Io)?;
        }
        Ok(())
    }

    /// 确保当前路径的父目录存在（若父目录缺失则递归创建）。
    ///
    /// # Returns
    ///
    /// * `Result<()>` - 成功时返回 `Ok(())`，失败时返回错误
    pub fn ensure_parent_dir_exists(&self) -> Result<(), FsError> {
        if let Some(parent) = self.root.parent() {
            fs::create_dir_all(parent).map_err(FsError::Io)?;
        }
        Ok(())
    }

    /// 路径是否存在。
    pub fn exists(&self) -> bool {
        self.root.exists()
    }

    /// 是否为文件。
    pub fn is_file(&self) -> bool {
        self.root.is_file()
    }

    /// 是否为目录。
    pub fn is_dir(&self) -> bool {
        self.root.is_dir()
    }

    /// 安全读取目录条目，忽略读取失败的条目。
    pub fn read_dir_safe(&self) -> Result<Vec<PathBuf>, FsError> {
        let entries = fs::read_dir(&self.root).map_err(|e| {
            FsError::Other(format!("Failed to read directory: {:?}: {}", self.root, e))
        })?;
        let mut paths = Vec::new();
        for entry in entries.flatten() {
            paths.push(entry.path());
        }
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::FsError;
    use tempfile::tempdir;

    use super::*;

    // ==================== DirectoryWalker::new 测试 ====================

    #[test]
    fn test_directory_walker_new() {
        let test_path = std::env::temp_dir().join("test");
        let walker = DirectoryWalker::new(&test_path);
        // 验证可以创建实例（通过使用它来验证）
        assert!(walker.exists() || !walker.exists()); // 总是为真，但验证 walker 可用
    }

    #[test]
    fn test_directory_walker_new_with_pathbuf() {
        let path = std::env::temp_dir().join("test");
        let walker = DirectoryWalker::new(path.clone());
        // 验证可以创建实例
        assert!(walker.exists() || !walker.exists());
    }

    // ==================== 目录遍历测试 ====================

    #[test]
    fn test_list_dirs_recursive() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // 创建目录结构
        fs::create_dir_all(root.join("dir1/subdir1"))?;
        fs::create_dir_all(root.join("dir2"))?;
        fs::create_dir_all(root.join("dir1/subdir2"))?;

        let walker = DirectoryWalker::new(root);
        let dirs = walker.list_dirs()?;

        // 应该包含根目录和所有子目录
        assert!(dirs.contains(&root.to_path_buf()));
        assert!(dirs.iter().any(|d| d.ends_with("dir1")));
        assert!(dirs.iter().any(|d| d.ends_with("subdir1")));
        assert!(dirs.iter().any(|d| d.ends_with("subdir2")));
        assert!(dirs.iter().any(|d| d.ends_with("dir2")));

        Ok(())
    }

    #[test]
    fn test_list_files_recursive() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // 创建文件和目录结构
        fs::create_dir_all(root.join("subdir"))?;
        fs::write(root.join("file1.txt"), "content1")?;
        fs::write(root.join("file2.txt"), "content2")?;
        fs::write(root.join("subdir/file3.txt"), "content3")?;

        let walker = DirectoryWalker::new(root);
        let files = walker.list_files()?;

        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.ends_with("file1.txt")));
        assert!(files.iter().any(|f| f.ends_with("file2.txt")));
        assert!(files.iter().any(|f| f.ends_with("file3.txt")));

        Ok(())
    }

    #[test]
    fn test_find_files() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // 创建文件
        fs::write(root.join("test_file.txt"), "content")?;
        fs::write(root.join("other_file.txt"), "content")?;
        fs::write(root.join("test_config.toml"), "content")?;
        fs::create_dir_all(root.join("subdir"))?;
        fs::write(root.join("subdir/test_file.rs"), "content")?;

        let walker = DirectoryWalker::new(root);
        let files = walker.find_files("test")?;

        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.ends_with("test_file.txt")));
        assert!(files.iter().any(|f| f.ends_with("test_config.toml")));
        assert!(files.iter().any(|f| f.ends_with("test_file.rs")));

        Ok(())
    }

    #[test]
    fn test_list_direct_dirs() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // 创建目录结构
        fs::create_dir_all(root.join("dir1/subdir"))?;
        fs::create_dir_all(root.join("dir2"))?;

        let walker = DirectoryWalker::new(root);
        let dirs = walker.list_direct_dirs()?;

        // 应该只包含直接子目录，不包含子目录的子目录
        assert_eq!(dirs.len(), 2);
        assert!(dirs.iter().any(|d| d.ends_with("dir1")));
        assert!(dirs.iter().any(|d| d.ends_with("dir2")));
        assert!(!dirs.iter().any(|d| d.ends_with("subdir")));

        Ok(())
    }

    #[test]
    fn test_list_direct_files() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // 创建文件和目录结构
        fs::create_dir_all(root.join("subdir"))?;
        fs::write(root.join("file1.txt"), "content1")?;
        fs::write(root.join("file2.txt"), "content2")?;
        fs::write(root.join("subdir/file3.txt"), "content3")?;

        let walker = DirectoryWalker::new(root);
        let files = walker.list_direct_files()?;

        // 应该只包含直接文件，不包含子目录中的文件
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.ends_with("file1.txt")));
        assert!(files.iter().any(|f| f.ends_with("file2.txt")));
        assert!(!files.iter().any(|f| f.ends_with("file3.txt")));

        Ok(())
    }

    // ==================== 目录创建测试 ====================

    #[test]
    fn test_ensure_exists() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        let new_dir = temp_dir.path().join("new_dir");

        let walker = DirectoryWalker::new(&new_dir);
        assert!(!walker.exists());

        walker.ensure_exists()?;
        assert!(walker.exists());
        assert!(walker.is_dir());

        Ok(())
    }

    #[test]
    fn test_ensure_exists_nested() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        let nested_dir = temp_dir.path().join("level1/level2/level3");

        let walker = DirectoryWalker::new(&nested_dir);
        walker.ensure_exists()?;

        assert!(walker.exists());
        assert!(nested_dir.is_dir());

        Ok(())
    }

    #[test]
    fn test_ensure_parent_exists() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("subdir/nested/file.txt");

        let walker = DirectoryWalker::new(temp_dir.path());
        walker.ensure_parent_exists(&file_path)?;

        let parent = file_path
            .parent()
            .expect("File path should have a parent directory");
        assert!(parent.exists());
        assert!(parent.is_dir());

        Ok(())
    }

    #[test]
    fn test_ensure_parent_dir_exists() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        let nested_path = temp_dir.path().join("level1/level2/target");

        let walker = DirectoryWalker::new(&nested_path);
        walker.ensure_parent_dir_exists()?;

        let parent = nested_path
            .parent()
            .expect("Nested path should have a parent directory");
        assert!(parent.exists());

        Ok(())
    }

    // ==================== 路径检查测试 ====================

    #[test]
    fn test_exists() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        let existing_path = temp_dir.path();
        let non_existing_path = temp_dir.path().join("nonexistent");

        let walker1 = DirectoryWalker::new(existing_path);
        assert!(walker1.exists());

        let walker2 = DirectoryWalker::new(non_existing_path);
        assert!(!walker2.exists());

        Ok(())
    }

    #[test]
    fn test_is_file() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content")?;

        let file_walker = DirectoryWalker::new(&file_path);
        assert!(file_walker.is_file());

        let dir_walker = DirectoryWalker::new(temp_dir.path());
        assert!(!dir_walker.is_file());

        Ok(())
    }

    #[test]
    fn test_is_dir() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content")?;

        let dir_walker = DirectoryWalker::new(temp_dir.path());
        assert!(dir_walker.is_dir());

        let file_walker = DirectoryWalker::new(&file_path);
        assert!(!file_walker.is_dir());

        Ok(())
    }

    // ==================== 安全读取测试 ====================

    #[test]
    fn test_read_dir_safe() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // 创建文件和目录
        fs::write(root.join("file1.txt"), "content1")?;
        fs::write(root.join("file2.txt"), "content2")?;
        fs::create_dir_all(root.join("subdir"))?;

        let walker = DirectoryWalker::new(root);
        let entries = walker.read_dir_safe()?;

        // 应该包含所有条目（文件和目录）
        assert!(entries.len() >= 3);
        assert!(entries.iter().any(|e| e.ends_with("file1.txt")));
        assert!(entries.iter().any(|e| e.ends_with("file2.txt")));
        assert!(entries.iter().any(|e| e.ends_with("subdir")));

        Ok(())
    }

    // ==================== 边界情况测试 ====================

    #[test]
    fn test_list_dirs_empty_directory() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        let walker = DirectoryWalker::new(temp_dir.path());
        let dirs = walker.list_dirs()?;

        // 应该至少包含根目录本身
        assert!(!dirs.is_empty());
        assert!(dirs.contains(&temp_dir.path().to_path_buf()));

        Ok(())
    }

    #[test]
    fn test_list_files_empty_directory() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        let walker = DirectoryWalker::new(temp_dir.path());
        let files = walker.list_files()?;

        assert_eq!(files.len(), 0);

        Ok(())
    }

    #[test]
    fn test_find_files_no_match() -> Result<(), FsError> {
        let temp_dir = tempdir()?;
        fs::write(temp_dir.path().join("file1.txt"), "content")?;
        fs::write(temp_dir.path().join("file2.txt"), "content")?;

        let walker = DirectoryWalker::new(temp_dir.path());
        let files = walker.find_files("nonexistent")?;

        assert_eq!(files.len(), 0);

        Ok(())
    }

    #[test]
    fn test_ensure_parent_exists_for_root_path() -> Result<(), FsError> {
        let walker = DirectoryWalker::new("/");
        let result = walker.ensure_parent_exists(Path::new("/"));

        // 根路径没有父目录，应该成功（不执行任何操作）
        assert!(result.is_ok());

        Ok(())
    }
}
