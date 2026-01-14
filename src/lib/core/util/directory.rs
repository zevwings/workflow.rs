//! 目录管理工具
//!
//! 提供基于路径的目录管理助手 `DirectoryWalker`，包括目录遍历、创建和路径检查功能。

use color_eyre::{eyre::WrapErr, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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
    pub fn list_dirs(&self) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();
        for entry in WalkDir::new(&self.root) {
            let entry = entry
                .wrap_err_with(|| format!("Failed to read directory entry: {:?}", self.root))?;
            if entry.file_type().is_dir() {
                dirs.push(entry.path().to_path_buf());
            }
        }
        Ok(dirs)
    }

    /// 递归列出所有文件。
    pub fn list_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for entry in WalkDir::new(&self.root) {
            let entry = entry
                .wrap_err_with(|| format!("Failed to read directory entry: {:?}", self.root))?;
            if entry.file_type().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }
        Ok(files)
    }

    /// 递归查找匹配模式的文件（文件名包含给定模式）。
    pub fn find_files(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for entry in WalkDir::new(&self.root) {
            let entry = entry
                .wrap_err_with(|| format!("Failed to read directory entry: {:?}", self.root))?;
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
    pub fn list_direct_dirs(&self) -> Result<Vec<PathBuf>> {
        let entries = fs::read_dir(&self.root)
            .wrap_err_with(|| format!("Failed to read directory: {:?}", self.root))?;
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
    pub fn list_direct_files(&self) -> Result<Vec<PathBuf>> {
        let entries = fs::read_dir(&self.root)
            .wrap_err_with(|| format!("Failed to read directory: {:?}", self.root))?;
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
    /// use workflow::util::directory::DirectoryWalker;
    /// # use color_eyre::Result;
    ///
    /// # fn main() -> Result<()> {
    /// let walker = DirectoryWalker::new("./test_dir");
    /// walker.ensure_exists()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn ensure_exists(&self) -> Result<()> {
        fs::create_dir_all(&self.root)
            .wrap_err_with(|| format!("Failed to create directory: {:?}", self.root))
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
    /// use workflow::util::directory::DirectoryWalker;
    /// use std::path::Path;
    /// # use color_eyre::Result;
    ///
    /// # fn main() -> Result<()> {
    /// let walker = DirectoryWalker::new(".");
    /// walker.ensure_parent_exists(Path::new("./some/nested/file.txt"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn ensure_parent_exists(&self, file_path: &Path) -> Result<()> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .wrap_err_with(|| format!("Failed to create parent directory: {:?}", parent))?;
        }
        Ok(())
    }

    /// 确保当前路径的父目录存在（若父目录缺失则递归创建）。
    ///
    /// # Returns
    ///
    /// * `Result<()>` - 成功时返回 `Ok(())`，失败时返回错误
    pub fn ensure_parent_dir_exists(&self) -> Result<()> {
        if let Some(parent) = self.root.parent() {
            fs::create_dir_all(parent)
                .wrap_err_with(|| format!("Failed to create parent directory: {:?}", parent))?;
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
    pub fn read_dir_safe(&self) -> Result<Vec<PathBuf>> {
        let entries = fs::read_dir(&self.root)
            .wrap_err_with(|| format!("Failed to read directory: {:?}", self.root))?;
        let mut paths = Vec::new();
        for entry in entries.flatten() {
            paths.push(entry.path());
        }
        Ok(paths)
    }
}
