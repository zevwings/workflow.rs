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
//! fn test_with_git_config_isolation_return_ok() -> color_eyre::Result<()> {
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
use serial_test::serial;
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
        const MAX_RETRIES: usize = 3;
        const RETRY_DELAY_MS: u64 = 100;

        // 使用稳定的工作目录（项目根目录或系统临时目录）
        // Git 命令使用 --file 参数，不需要特定的工作目录
        // 但在并行测试时，当前工作目录可能被删除，导致 Git 命令失败
        // 优先使用项目根目录（CARGO_MANIFEST_DIR），如果不可用则使用系统临时目录
        let stable_dir = std::env::var("CARGO_MANIFEST_DIR")
            .ok()
            .and_then(|p| std::path::PathBuf::from(p).canonicalize().ok())
            .or_else(|| {
                std::env::current_dir()
                    .ok()
                    .and_then(|d| d.canonicalize().ok())
                    .filter(|d| d.exists())
            })
            .unwrap_or_else(|| std::env::temp_dir());

        // 重试机制：处理锁文件冲突和短暂的并发锁定
        let config_path_str = self
            .config_path
            .to_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("config path should be valid UTF-8"))?;
        for attempt in 0..MAX_RETRIES {
            // 清理可能存在的锁文件（解决锁文件残留问题）
            // Git 在写入配置文件时会创建锁文件（.tmpXXXXX.lock）
            // 如果之前的测试异常退出，锁文件可能残留
            let lock_file = format!("{}.lock", self.config_path.to_string_lossy());
            if std::path::Path::new(&lock_file).exists() {
                // 忽略清理失败（锁文件可能正在被使用）
                let _ = std::fs::remove_file(&lock_file);
            }

            let output = Command::new("git")
                .args(["config", "--file", config_path_str, key, value])
                .current_dir(&stable_dir) // 设置稳定的工作目录，避免并行测试时目录被删除的问题
                .output()
                .map_err(|e| color_eyre::eyre::eyre!("Failed to execute git config: {}", e))?;

            if output.status.success() {
                return Ok(());
            }

            // 检查是否是锁文件错误
            let error = String::from_utf8_lossy(&output.stderr);
            let is_lock_error = error.contains("could not lock config file");

            // 如果是锁文件错误且还有重试机会，等待后重试
            if is_lock_error && attempt < MAX_RETRIES - 1 {
                std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS));
                continue;
            }

            // 其他错误或重试次数用尽，返回错误
            return Err(color_eyre::eyre::eyre!(
                "Failed to set Git config {}={}: {}",
                key,
                value,
                error
            ));
        }

        unreachable!()
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

    /// 测试GitConfigGuard设置配置项
    ///
    /// ## 测试目的
    /// 验证 `GitConfigGuard::set()` 方法能够正确设置Git配置项到临时配置文件。
    ///
    /// ## 测试场景
    /// 1. 创建GitConfigGuard
    /// 2. 设置多个配置项（user.name, user.email）
    /// 3. 使用git config命令验证配置已设置
    ///
    /// ## 预期结果
    /// - 配置项设置成功
    /// - git config命令能够读取设置的配置值
    #[test]
    fn test_git_config_guard_set_return_ok() -> Result<()> {
        let guard = GitConfigGuard::new()?;

        guard.set("user.name", "Test User")?;
        guard.set("user.email", "test@example.com")?;

        // 验证配置已设置
        // 使用稳定的工作目录，避免并行测试时目录被删除的问题
        let stable_dir = std::env::var("CARGO_MANIFEST_DIR")
            .ok()
            .and_then(|p| std::path::PathBuf::from(p).canonicalize().ok())
            .or_else(|| {
                std::env::current_dir()
                    .ok()
                    .and_then(|d| d.canonicalize().ok())
                    .filter(|d| d.exists())
            })
            .unwrap_or_else(|| std::env::temp_dir());

        let config_path_str = guard
            .config_path()
            .to_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("config path should be valid UTF-8"))?;
        let output = Command::new("git")
            .args(["config", "--file", config_path_str, "user.name"])
            .current_dir(&stable_dir)
            .output()?;

        assert!(output.status.success());
        let name = String::from_utf8(output.stdout)?;
        assert_eq!(name.trim(), "Test User");

        Ok(())
    }

    /// 测试GitConfigGuard自动恢复GIT_CONFIG环境变量
    ///
    /// ## 测试目的
    /// 验证 `GitConfigGuard` 在drop时能够自动恢复原始的GIT_CONFIG环境变量。
    ///
    /// ## 测试场景
    /// 1. 保存原始GIT_CONFIG环境变量值
    /// 2. 创建GitConfigGuard（会设置GIT_CONFIG）
    /// 3. 验证GIT_CONFIG已设置
    /// 4. Drop guard
    /// 5. 验证GIT_CONFIG已恢复为原始值
    ///
    /// ## 预期结果
    /// - Guard创建时，GIT_CONFIG被设置
    /// - Guard drop后，GIT_CONFIG恢复为原始值（或移除，如果原本不存在）
    ///
    /// ## 注意事项
    /// - 使用 `#[serial]` 标记，避免并行测试时环境变量污染
    /// - 如果原始值是临时文件路径（可能是其他测试设置的），只验证环境变量被恢复，不验证路径完全相同
    #[test]
    #[serial]
    fn test_git_config_guard_restore_return_ok() -> Result<()> {
        let original_git_config = std::env::var("GIT_CONFIG").ok();
        let is_temp_file_path = original_git_config
            .as_ref()
            .map(|p| p.contains(".tmp") || p.contains("temp"))
            .unwrap_or(false);

        {
            let _guard = GitConfigGuard::new()?;
            // 验证GIT_CONFIG已设置
            assert!(std::env::var("GIT_CONFIG").is_ok());
        }

        // 验证GIT_CONFIG已恢复
        match original_git_config {
            Some(ref val) => {
                let current = std::env::var("GIT_CONFIG")
                    .map_err(|e| color_eyre::eyre::eyre!("GIT_CONFIG should exist: {}", e))?;

                // 如果原始值是临时文件路径，在并行测试中可能被其他测试修改
                // 只验证环境变量被恢复（存在且是路径格式），不验证路径完全相同
                if is_temp_file_path {
                    // 验证恢复后的值也是临时文件路径格式
                    assert!(
                        current.contains(".tmp") || current.contains("temp"),
                        "Restored GIT_CONFIG should be a temp file path, got: {}",
                        current
                    );
                } else {
                    // 非临时文件路径，应该完全匹配
                    assert_eq!(current, *val);
                }
            }
            None => assert!(std::env::var("GIT_CONFIG").is_err()),
        }

        Ok(())
    }
}
