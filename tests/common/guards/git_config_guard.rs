//! Git配置隔离守卫
//!
//! 临时修改Git配置，测试结束后自动恢复。
//!
//! # 使用示例
//!
//! ```rust
//! use tests::common::guards::GitConfigGuard;
//!
//! #[test]
//! fn test_with_git_config_isolation() -> color_eyre::Result<()> {
//!     let guard = GitConfigGuard::new()?;
//!
//!     // 设置Git配置项
//!     guard.set("user.name", "Test User")?;
//!     guard.set("user.email", "test@example.com")?;
//!
//!     // 测试代码...
//!
//!     // Drop时自动恢复Git配置
//!     Ok(())
//! }
//! ```

use color_eyre::Result;
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

/// Git配置隔离守卫
///
/// 通过设置`GIT_CONFIG`环境变量指向临时配置文件，实现Git配置的隔离。
/// 测试结束后自动恢复原始的`GIT_CONFIG`环境变量。
///
/// # 功能特性
///
/// - ✅ RAII模式自动清理
/// - ✅ 使用临时配置文件隔离
/// - ✅ 自动恢复原始GIT_CONFIG环境变量
/// - ✅ 支持从全局配置复制
pub struct GitConfigGuard {
    /// 临时Git配置文件
    _temp_config_file: NamedTempFile,
    /// 原始的GIT_CONFIG环境变量值
    original_git_config_env: Option<String>,
    /// 临时配置文件路径
    config_path: PathBuf,
}

impl GitConfigGuard {
    /// 创建独立的Git配置环境
    ///
    /// 创建一个临时Git配置文件，并设置`GIT_CONFIG`环境变量指向它。
    ///
    /// # 返回
    ///
    /// 成功时返回`GitConfigGuard`实例，失败时返回错误
    ///
    /// # 错误
    ///
    /// - 无法创建临时文件
    /// - 无法获取临时文件路径
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::guards::GitConfigGuard;
    ///
    /// let guard = GitConfigGuard::new()?;
    /// ```
    pub fn new() -> Result<Self> {
        // 创建临时配置文件
        let temp_file = tempfile::NamedTempFile::new()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create temp Git config file: {}", e))?;

        let config_path = temp_file.path().to_path_buf();

        // 保存原始的GIT_CONFIG环境变量
        let original_git_config_env = std::env::var("GIT_CONFIG").ok();

        // 设置GIT_CONFIG环境变量指向临时文件
        std::env::set_var("GIT_CONFIG", config_path.to_string_lossy().as_ref());

        Ok(Self {
            _temp_config_file: temp_file,
            original_git_config_env,
            config_path,
        })
    }

    /// 设置Git配置项
    ///
    /// 使用`git config`命令设置配置项到临时配置文件。
    ///
    /// # 参数
    ///
    /// * `key` - Git配置键（如 "user.name"）
    /// * `value` - Git配置值
    ///
    /// # 返回
    ///
    /// 成功时返回`Ok(())`，失败时返回错误
    ///
    /// # 错误
    ///
    /// - Git命令执行失败
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::guards::GitConfigGuard;
    ///
    /// let guard = GitConfigGuard::new()?;
    /// guard.set("user.name", "Test User")?;
    /// guard.set("user.email", "test@example.com")?;
    /// ```
    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        let output = Command::new("git")
            .args([
                "config",
                "--file",
                self.config_path.to_str().unwrap(),
                key,
                value,
            ])
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to execute git config: {}", e))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!(
                "Failed to set Git config {}={}: {}",
                key,
                value,
                error
            ));
        }

        Ok(())
    }

    /// 从全局配置复制
    ///
    /// 将全局Git配置复制到临时配置文件。
    /// 这对于需要保留某些全局配置的测试很有用。
    ///
    /// # 返回
    ///
    /// 成功时返回`Ok(())`，失败时返回错误
    ///
    /// # 错误
    ///
    /// - Git命令执行失败
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::guards::GitConfigGuard;
    ///
    /// let guard = GitConfigGuard::new()?;
    /// guard.copy_from_global()?;
    /// guard.set("user.name", "Test User")?; // 覆盖特定配置
    /// ```
    #[allow(unused)]
    pub fn copy_from_global(&self) -> Result<()> {
        // 获取全局配置路径
        let global_config = dirs::home_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get home directory"))?
            .join(".gitconfig");

        if global_config.exists() {
            // 复制全局配置到临时文件
            std::fs::copy(&global_config, &self.config_path)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to copy global Git config: {}", e))?;
        }

        Ok(())
    }

    /// 获取配置文件路径（用于调试）
    ///
    /// # 返回
    ///
    /// 返回临时配置文件的路径
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}

impl Drop for GitConfigGuard {
    fn drop(&mut self) {
        // 恢复原始的GIT_CONFIG环境变量
        match &self.original_git_config_env {
            Some(value) => {
                std::env::set_var("GIT_CONFIG", value);
            }
            None => {
                std::env::remove_var("GIT_CONFIG");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_config_guard_set() -> Result<()> {
        let guard = GitConfigGuard::new()?;

        guard.set("user.name", "Test User")?;
        guard.set("user.email", "test@example.com")?;

        // 验证配置已设置
        let output = Command::new("git")
            .args([
                "config",
                "--file",
                guard.config_path().to_str().unwrap(),
                "user.name",
            ])
            .output()?;

        assert!(output.status.success());
        let name = String::from_utf8(output.stdout)?;
        assert_eq!(name.trim(), "Test User");

        Ok(())
    }

    #[test]
    fn test_git_config_guard_restore() -> Result<()> {
        let original_git_config = std::env::var("GIT_CONFIG").ok();

        {
            let _guard = GitConfigGuard::new()?;
            // 验证GIT_CONFIG已设置
            assert!(std::env::var("GIT_CONFIG").is_ok());
        }

        // 验证GIT_CONFIG已恢复
        match original_git_config {
            Some(ref val) => assert_eq!(std::env::var("GIT_CONFIG").unwrap(), *val),
            None => assert!(std::env::var("GIT_CONFIG").is_err()),
        }

        Ok(())
    }
}
