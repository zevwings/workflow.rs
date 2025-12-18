//! 路径操作工具
//!
//! 提供基于路径的助手 `PathAccess`，封装常用的目录/文件检查与创建。

use crate::base::util::directory::DirectoryWalker;
use color_eyre::{eyre::WrapErr, Result};
use std::fs;
use std::path::PathBuf;

/// 路径助手，封装常用的目录/文件操作。
pub struct PathAccess {
    path: PathBuf,
}

impl PathAccess {
    /// 创建新的路径助手。
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// 确保目录存在（若不存在则递归创建）。
    pub fn ensure_dir_exists(&self) -> Result<()> {
        if !self.path.exists() {
            DirectoryWalker::new(&self.path).ensure_exists()?;
        }
        Ok(())
    }

    /// 确保父目录存在（若父目录缺失则递归创建）。
    pub fn ensure_parent_exists(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            PathAccess::new(parent).ensure_dir_exists()?;
        }
        Ok(())
    }

    /// 安全读取目录条目，忽略读取失败的条目。
    pub fn read_dir_safe(&self) -> Result<Vec<PathBuf>> {
        let entries = fs::read_dir(&self.path)
            .wrap_err_with(|| format!("Failed to read directory: {:?}", self.path))?;
        let mut paths = Vec::new();
        for entry in entries.flatten() {
            paths.push(entry.path());
        }
        Ok(paths)
    }

    /// 路径是否存在。
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// 是否为文件。
    pub fn is_file(&self) -> bool {
        self.path.is_file()
    }

    /// 是否为目录。
    pub fn is_dir(&self) -> bool {
        self.path.is_dir()
    }
}
