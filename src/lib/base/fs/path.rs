//! 路径操作工具
//!
//! 提供基于路径的助手 `PathAccess`，封装常用的目录/文件检查与创建。

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
            fs::create_dir_all(&self.path)
                .wrap_err_with(|| format!("Failed to create directory: {:?}", self.path))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_basic_operations() -> Result<()> {
        // Basic validation of core PathAccess functionality
        let temp_dir = TempDir::new().unwrap();

        // Test exists/is_file/is_dir
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();
        let path_access = PathAccess::new(&file_path);
        assert!(path_access.exists());
        assert!(path_access.is_file());

        // Test ensure_dir_exists
        let new_dir = temp_dir.path().join("new/dir");
        let dir_access = PathAccess::new(&new_dir);
        dir_access.ensure_dir_exists()?;
        assert!(new_dir.exists() && new_dir.is_dir());

        Ok(())
    }
}
