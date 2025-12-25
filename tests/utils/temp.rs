//! 临时文件管理工具模块
//!
//! 提供统一的临时文件和临时目录管理功能，确保资源的正确清理。
//!
//! 本模块专门用于测试代码，提供测试中临时文件和目录的管理功能。

use color_eyre::{eyre::WrapErr, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

/// 临时文件管理器
///
/// 提供临时文件和目录的创建、管理和清理功能。
/// 实现了 Drop trait 以确保临时资源的自动清理。
#[derive(Debug)]
pub struct TempManager {
    /// 临时目录
    temp_dir: TempDir,
    /// 创建的临时文件列表
    temp_files: Vec<PathBuf>,
}

impl TempManager {
    /// 创建新的临时文件管理器
    ///
    /// # 返回值
    ///
    /// 返回 `Result<TempManager>`，包含新创建的临时文件管理器
    ///
    /// # 错误
    ///
    /// 当无法创建临时目录时返回错误
    ///
    /// # 示例
    ///
    /// ```
    /// use utils::temp::TempManager;
    ///
    /// let temp_manager = TempManager::new()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new() -> Result<Self> {
        let temp_dir = tempdir().with_context(|| "Failed to create temporary directory")?;

        Ok(Self {
            temp_dir,
            temp_files: Vec::new(),
        })
    }

    /// 获取临时目录路径
    ///
    /// # 返回值
    ///
    /// 返回临时目录的路径引用
    ///
    /// # 示例
    ///
    /// ```
    /// use utils::temp::TempManager;
    ///
    /// let temp_manager = TempManager::new()?;
    /// let temp_path = temp_manager.temp_dir();
    /// println!("Temp directory: {}", temp_path.display());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn temp_dir(&self) -> &Path {
        self.temp_dir.path()
    }

    /// 在临时目录中创建文件
    ///
    /// # 参数
    ///
    /// * `filename` - 文件名
    /// * `content` - 文件内容
    ///
    /// # 返回值
    ///
    /// 返回 `Result<PathBuf>`，包含创建的文件路径
    ///
    /// # 错误
    ///
    /// 当无法创建或写入文件时返回错误
    ///
    /// # 示例
    ///
    /// ```
    /// use utils::temp::TempManager;
    ///
    /// let mut temp_manager = TempManager::new()?;
    /// let file_path = temp_manager.create_file("test.txt", "Hello, World!")?;
    /// assert!(file_path.exists());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create_file(&mut self, filename: &str, content: &str) -> Result<PathBuf> {
        let file_path = self.temp_dir.path().join(filename);

        fs::write(&file_path, content)
            .with_context(|| format!("Failed to write temporary file: {}", file_path.display()))?;

        self.temp_files.push(file_path.clone());
        Ok(file_path)
    }

    /// 在临时目录中创建子目录
    ///
    /// # 参数
    ///
    /// * `dirname` - 目录名
    ///
    /// # 返回值
    ///
    /// 返回 `Result<PathBuf>`，包含创建的目录路径
    ///
    /// # 错误
    ///
    /// 当无法创建目录时返回错误
    ///
    /// # 示例
    ///
    /// ```
    /// use utils::temp::TempManager;
    ///
    /// let temp_manager = TempManager::new()?;
    /// let dir_path = temp_manager.create_dir("subdir")?;
    /// assert!(dir_path.is_dir());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create_dir(&self, dirname: &str) -> Result<PathBuf> {
        let dir_path = self.temp_dir.path().join(dirname);

        fs::create_dir_all(&dir_path).with_context(|| {
            format!(
                "Failed to create temporary directory: {}",
                dir_path.display()
            )
        })?;

        Ok(dir_path)
    }

    /// 获取临时文件路径
    ///
    /// # 参数
    ///
    /// * `filename` - 文件名
    ///
    /// # 返回值
    ///
    /// 返回临时文件的完整路径
    ///
    /// # 示例
    ///
    /// ```
    /// use utils::temp::TempManager;
    ///
    /// let temp_manager = TempManager::new()?;
    /// let file_path = temp_manager.file_path("config.toml");
    /// println!("File path: {}", file_path.display());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[allow(dead_code)]
    pub fn file_path(&self, filename: &str) -> PathBuf {
        self.temp_dir.path().join(filename)
    }

    /// 清理指定的临时文件
    ///
    /// # 参数
    ///
    /// * `file_path` - 要清理的文件路径
    ///
    /// # 返回值
    ///
    /// 返回 `Result<()>`
    ///
    /// # 错误
    ///
    /// 当无法删除文件时返回错误
    ///
    /// # 示例
    ///
    /// ```
    /// use utils::temp::TempManager;
    ///
    /// let mut temp_manager = TempManager::new()?;
    /// let file_path = temp_manager.create_file("temp.txt", "content")?;
    /// temp_manager.cleanup_file(&file_path)?;
    /// assert!(!file_path.exists());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn cleanup_file(&mut self, file_path: &Path) -> Result<()> {
        if file_path.exists() {
            fs::remove_file(file_path).with_context(|| {
                format!("Failed to cleanup temporary file: {}", file_path.display())
            })?;
        }

        // 从跟踪列表中移除
        self.temp_files.retain(|p| p != file_path);

        Ok(())
    }

    /// 清理所有创建的临时文件
    ///
    /// # 返回值
    ///
    /// 返回 `Result<()>`
    ///
    /// # 错误
    ///
    /// 当无法删除某些文件时返回错误
    ///
    /// # 示例
    ///
    /// ```
    /// use utils::temp::TempManager;
    ///
    /// let mut temp_manager = TempManager::new()?;
    /// temp_manager.create_file("file1.txt", "content1")?;
    /// temp_manager.create_file("file2.txt", "content2")?;
    /// temp_manager.cleanup_all_files()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn cleanup_all_files(&mut self) -> Result<()> {
        let mut errors = Vec::new();

        for file_path in &self.temp_files {
            if file_path.exists() {
                if let Err(e) = fs::remove_file(file_path) {
                    errors.push(format!("Failed to cleanup {}: {}", file_path.display(), e));
                }
            }
        }

        self.temp_files.clear();

        if !errors.is_empty() {
            return Err(color_eyre::eyre::eyre!(
                "Failed to cleanup some temporary files: {}",
                errors.join(", ")
            ));
        }

        Ok(())
    }

    /// 获取已创建的临时文件列表
    ///
    /// # 返回值
    ///
    /// 返回临时文件路径的引用列表
    pub fn temp_files(&self) -> &[PathBuf] {
        &self.temp_files
    }
}

impl Drop for TempManager {
    /// 自动清理资源
    ///
    /// 当 TempManager 被销毁时，自动清理所有创建的临时文件。
    /// 临时目录会由 TempDir 自动清理。
    fn drop(&mut self) {
        // 尝试清理所有临时文件，忽略错误（因为在 Drop 中无法处理错误）
        let _ = self.cleanup_all_files();
    }
}

/// 在临时目录中执行操作的便捷函数
///
/// # 参数
///
/// * `operation` - 要执行的操作闭包，接收临时目录路径作为参数
///
/// # 返回值
///
/// 返回操作的结果
///
/// # 错误
///
/// 当无法创建临时目录或操作失败时返回错误
///
/// # 示例
///
/// ```
/// use utils::temp::with_temp_dir;
/// use std::fs;
///
/// let result = with_temp_dir(|temp_path| {
///     let file_path = temp_path.join("test.txt");
///     fs::write(&file_path, "Hello, World!")?;
///     fs::read_to_string(&file_path)
/// })?;
///
/// assert_eq!(result, "Hello, World!");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn with_temp_dir<F, R>(operation: F) -> Result<R>
where
    F: FnOnce(&Path) -> Result<R>,
{
    let temp_manager = TempManager::new()?;
    operation(temp_manager.temp_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试TempManager创建
    ///
    /// ## 测试目的
    /// 验证 `TempManager::new()` 能够成功创建临时文件管理器，包括临时目录的创建。
    ///
    /// ## 测试场景
    /// 1. 创建TempManager实例
    /// 2. 获取临时目录路径
    /// 3. 验证临时目录存在且为目录
    ///
    /// ## 预期结果
    /// - 临时目录存在
    /// - 临时目录为目录类型
    #[test]
    fn test_temp_manager_creation() -> Result<()> {
        let temp_manager = TempManager::new()?;
        assert!(temp_manager.temp_dir().exists());
        assert!(temp_manager.temp_dir().is_dir());
        Ok(())
    }

    /// 测试创建文件
    ///
    /// ## 测试目的
    /// 验证 `TempManager::create_file()` 方法能够成功创建文件，并写入指定内容。
    ///
    /// ## 测试场景
    /// 1. 创建TempManager
    /// 2. 创建文件（test.txt）
    /// 3. 验证文件存在且为文件类型
    /// 4. 验证文件内容正确
    ///
    /// ## 预期结果
    /// - 文件创建成功
    /// - 文件存在且为文件类型
    /// - 文件内容与预期一致
    #[test]
    fn test_create_file() -> Result<()> {
        let mut temp_manager = TempManager::new()?;
        let file_path = temp_manager.create_file("test.txt", "Hello, World!")?;

        assert!(file_path.exists());
        assert!(file_path.is_file());

        let content = fs::read_to_string(&file_path)?;
        assert_eq!(content, "Hello, World!");

        Ok(())
    }

    /// 测试创建目录
    ///
    /// ## 测试目的
    /// 验证 `TempManager::create_dir()` 方法能够成功创建子目录。
    ///
    /// ## 测试场景
    /// 1. 创建TempManager
    /// 2. 创建子目录（subdir）
    /// 3. 验证目录存在且为目录类型
    ///
    /// ## 预期结果
    /// - 目录创建成功
    /// - 目录存在且为目录类型
    #[test]
    fn test_create_dir() -> Result<()> {
        let temp_manager = TempManager::new()?;
        let dir_path = temp_manager.create_dir("subdir")?;

        assert!(dir_path.exists());
        assert!(dir_path.is_dir());

        Ok(())
    }

    /// 测试清理单个文件
    ///
    /// ## 测试目的
    /// 验证 `TempManager::cleanup_file()` 方法能够成功删除指定的临时文件。
    ///
    /// ## 测试场景
    /// 1. 创建TempManager并创建文件
    /// 2. 验证文件存在
    /// 3. 调用cleanup_file删除文件
    /// 4. 验证文件已删除
    ///
    /// ## 预期结果
    /// - 文件删除成功
    /// - 文件不再存在
    #[test]
    fn test_cleanup_file() -> Result<()> {
        let mut temp_manager = TempManager::new()?;
        let file_path = temp_manager.create_file("temp.txt", "content")?;

        assert!(file_path.exists());
        temp_manager.cleanup_file(&file_path)?;
        assert!(!file_path.exists());

        Ok(())
    }

    /// 测试清理所有文件
    ///
    /// ## 测试目的
    /// 验证 `TempManager::cleanup_all_files()` 方法能够成功删除所有创建的临时文件。
    ///
    /// ## 测试场景
    /// 1. 创建TempManager并创建多个文件
    /// 2. 验证所有文件存在
    /// 3. 调用cleanup_all_files删除所有文件
    /// 4. 验证所有文件已删除
    /// 5. 验证临时文件列表为空
    ///
    /// ## 预期结果
    /// - 所有文件删除成功
    /// - 所有文件不再存在
    /// - 临时文件列表为空
    #[test]
    fn test_cleanup_all_files() -> Result<()> {
        let mut temp_manager = TempManager::new()?;
        let file1 = temp_manager.create_file("file1.txt", "content1")?;
        let file2 = temp_manager.create_file("file2.txt", "content2")?;

        assert!(file1.exists());
        assert!(file2.exists());

        temp_manager.cleanup_all_files()?;

        assert!(!file1.exists());
        assert!(!file2.exists());
        assert!(temp_manager.temp_files().is_empty());

        Ok(())
    }

    /// 测试with_temp_dir便捷函数
    ///
    /// ## 测试目的
    /// 验证 `with_temp_dir()` 便捷函数能够在临时目录中执行操作，并在操作完成后自动清理。
    ///
    /// ## 测试场景
    /// 1. 使用with_temp_dir创建临时目录
    /// 2. 在临时目录中创建文件并读取内容
    /// 3. 验证操作成功
    /// 4. 函数返回后临时目录自动清理
    ///
    /// ## 预期结果
    /// - 操作成功执行
    /// - 返回结果正确
    /// - 临时目录自动清理
    #[test]
    fn test_with_temp_dir() -> Result<()> {
        let result = with_temp_dir(|temp_path| {
            let file_path = temp_path.join("test.txt");
            fs::write(&file_path, "Hello, World!")?;
            Ok(fs::read_to_string(&file_path)?)
        })?;

        assert_eq!(result, "Hello, World!");
        Ok(())
    }
}
