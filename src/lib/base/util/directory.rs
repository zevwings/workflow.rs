//! 目录遍历工具
//!
//! 提供基于路径的目录遍历助手 `DirectoryWalker`，统一使用 `walkdir` 进行遍历。

use crate::base::util::path::PathAccess;
use color_eyre::{eyre::WrapErr, Result};
use std::path::PathBuf;
use walkdir::WalkDir;

/// 目录遍历助手，基于固定根路径提供常用操作。
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
        let entries = PathAccess::new(&self.root).read_dir_safe()?;
        Ok(entries.into_iter().filter(|p| p.is_dir()).collect())
    }

    /// 非递归列出直接文件。
    pub fn list_direct_files(&self) -> Result<Vec<PathBuf>> {
        let entries = PathAccess::new(&self.root).read_dir_safe()?;
        Ok(entries.into_iter().filter(|p| p.is_file()).collect())
    }
}
