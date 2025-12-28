#![allow(clippy::test_attr_in_doctest)]

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

use color_eyre::{eyre::WrapErr, Result};
use git2::Config;
use serial_test::serial;
use std::path::PathBuf;
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
    /// 临时Git配置文件（保持文件存活）
    #[allow(dead_code)] // 通过 .path() 方法访问，编译器无法检测到使用
    temp_config_file: NamedTempFile,
    /// 原始的GIT_CONFIG环境变量值
    original_git_config_env: Option<String>,
    /// 临时配置文件路径（Windows上使用规范化路径）
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
        // Windows 上使用更安全的临时文件创建方式
        #[cfg(target_os = "windows")]
        {
            // 在 Windows 上，使用 NamedTempFile 在临时目录中创建文件
            // 使用 .tmp 后缀确保文件能被正确识别为临时文件
            let temp_dir = std::env::temp_dir();
            let temp_file =
                tempfile::NamedTempFile::with_suffix_in(".tmp", &temp_dir).map_err(|e| {
                    color_eyre::eyre::eyre!("Failed to create temp Git config file: {}", e)
                })?;

            // 确保文件存在（创建空文件）
            // 这对于后续操作在 Windows 上正常工作很重要
            std::fs::write(temp_file.path(), "")
                .map_err(|e| color_eyre::eyre::eyre!("Failed to initialize config file: {}", e))?;

            // 在 Windows 上，使用 canonicalize() 获取长路径格式，避免短路径（8.3格式）问题
            // 短路径（如 RUNNER~1）在某些情况下可能无法正确解析
            let original_path = temp_file.path();
            let canonical_path = std::fs::canonicalize(original_path).map_err(|e| {
                color_eyre::eyre::eyre!(
                    "Failed to canonicalize config file path {}: {}",
                    original_path.display(),
                    e
                )
            })?;

            // 移除 \\?\ 前缀（如果存在）以确保 git2 和标准库兼容性
            // git2 和标准库可能不支持扩展路径前缀
            let config_path_str = canonical_path.to_string_lossy().to_string();
            let path_without_prefix = if config_path_str.starts_with("\\\\?\\") {
                PathBuf::from(&config_path_str[4..])
            } else {
                canonical_path.clone()
            };

            // 确定最终使用的路径：优先使用不带前缀的路径，如果不存在则尝试其他选项
            let final_config_path = if path_without_prefix.exists() {
                // 不带前缀的路径存在，使用它
                path_without_prefix
            } else if original_path.exists() {
                // 不带前缀的路径不存在，但原始路径存在，使用原始路径
                original_path.to_path_buf()
            } else if canonical_path.exists() {
                // 只有带前缀的路径存在，尝试使用它（git2 可能支持）
                canonical_path
            } else {
                // 所有路径都不存在，这不应该发生，但使用原始路径作为后备
                original_path.to_path_buf()
            };

            // 验证路径确实存在且可访问
            std::fs::metadata(&final_config_path).map_err(|e| {
                color_eyre::eyre::eyre!(
                    "Failed to access config file {} (tried: without_prefix={}, original={}, canonical={}): {}",
                    final_config_path.display(),
                    path_without_prefix.display(),
                    original_path.display(),
                    canonical_path.display(),
                    e
                )
            })?;

            let config_path = final_config_path;

            // 保存原始的GIT_CONFIG环境变量
            let original_git_config_env = std::env::var("GIT_CONFIG").ok();

            // 设置GIT_CONFIG环境变量指向临时文件（使用规范化后的长路径）
            std::env::set_var("GIT_CONFIG", config_path.to_string_lossy().as_ref());

            Ok(Self {
                temp_config_file: temp_file,
                original_git_config_env,
                config_path,
            })
        }

        #[cfg(not(target_os = "windows"))]
        {
            // 非 Windows 平台使用标准方式
            let temp_file = tempfile::NamedTempFile::new().map_err(|e| {
                color_eyre::eyre::eyre!("Failed to create temp Git config file: {}", e)
            })?;

            let config_path = temp_file.path().to_path_buf();

            // 保存原始的GIT_CONFIG环境变量
            let original_git_config_env = std::env::var("GIT_CONFIG").ok();

            // 设置GIT_CONFIG环境变量指向临时文件
            std::env::set_var("GIT_CONFIG", config_path.to_string_lossy().as_ref());

            Ok(Self {
                temp_config_file: temp_file,
                original_git_config_env,
                config_path,
            })
        }
    }

    /// 设置Git配置项
    ///
    /// 使用 git2 API 设置配置项到临时配置文件。
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
        // Windows 上需要更多的重试次数和更长的延迟
        #[cfg(target_os = "windows")]
        const MAX_RETRIES: usize = 10;
        #[cfg(not(target_os = "windows"))]
        const MAX_RETRIES: usize = 3;

        #[cfg(target_os = "windows")]
        const RETRY_DELAY_MS: u64 = 200; // Windows 上需要更长的延迟
        #[cfg(not(target_os = "windows"))]
        const RETRY_DELAY_MS: u64 = 100;

        // 重试机制：处理锁文件冲突和短暂的并发锁定
        for attempt in 0..MAX_RETRIES {
            // Windows 上需要清理所有可能的临时文件和锁文件
            #[cfg(target_os = "windows")]
            {
                // 清理锁文件（使用规范化后的路径）
                let lock_file = format!("{}.lock", self.config_path.to_string_lossy());
                if std::path::Path::new(&lock_file).exists() {
                    for _ in 0..3 {
                        if std::fs::remove_file(&lock_file).is_ok() {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                }

                // 清理 git2 创建的临时文件（.tmpXXXXX）
                // git2 在写入配置时会创建临时文件，如果操作失败可能残留
                if let Some(parent) = self.config_path.parent() {
                    if let Some(_file_name) = self.config_path.file_name() {
                        // 查找所有以 .tmp 开头的临时文件
                        if let Ok(entries) = std::fs::read_dir(parent) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if let Some(name) = path.file_name() {
                                    let name_str = name.to_string_lossy();
                                    // 匹配 git2 创建的临时文件模式（.tmp + 随机字符）
                                    if name_str.starts_with(".tmp") && name_str.len() > 4 {
                                        // 尝试删除临时文件（忽略失败，可能正在被使用）
                                        let _ = std::fs::remove_file(&path);
                                    }
                                }
                            }
                        }
                    }
                }

                // Windows 上需要额外等待，确保文件完全释放
                if attempt > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                // 清理可能存在的锁文件（解决锁文件残留问题）
                // Git 在写入配置文件时会创建锁文件（.tmpXXXXX.lock）
                // 如果之前的测试异常退出，锁文件可能残留
                let lock_file = format!("{}.lock", self.config_path.to_string_lossy());
                if std::path::Path::new(&lock_file).exists() {
                    // 忽略清理失败（锁文件可能正在被使用）
                    let _ = std::fs::remove_file(&lock_file);
                }
            }

            // 使用 git2 API 打开并设置配置
            // 统一使用规范化后的路径（Windows上已转换为长路径）
            let config_path = &self.config_path;

            let config_result = Config::open(config_path);
            match config_result {
                Ok(mut config) => {
                    // Windows 上先检查文件权限和可访问性（使用规范化后的路径）
                    #[cfg(target_os = "windows")]
                    {
                        // 确保文件存在且可写（使用规范化后的长路径）
                        match std::fs::metadata(config_path) {
                            Ok(metadata) => {
                                // 检查文件是否可写（通过尝试打开文件）
                                if let Err(e) =
                                    std::fs::OpenOptions::new().write(true).open(config_path)
                                {
                                    drop(config);
                                    if attempt < MAX_RETRIES - 1 {
                                        std::thread::sleep(std::time::Duration::from_millis(
                                            RETRY_DELAY_MS * 2,
                                        ));
                                        continue;
                                    }
                                    return Err(color_eyre::eyre::eyre!(
                                        "Config file is not writable: {} (metadata: {:?})",
                                        e,
                                        metadata
                                    ));
                                }
                            }
                            Err(e) => {
                                drop(config);
                                // On Windows, file might not be immediately accessible after creation
                                // Retry with exponential backoff
                                if attempt < MAX_RETRIES - 1 {
                                    std::thread::sleep(std::time::Duration::from_millis(
                                        RETRY_DELAY_MS * 2,
                                    ));
                                    continue;
                                }
                                // Provide more context about the error
                                let file_exists = config_path.exists();
                                return Err(color_eyre::eyre::eyre!(
                                    "Failed to access config file {}: {} (file exists: {})",
                                    config_path.display(),
                                    e,
                                    file_exists
                                ));
                            }
                        }
                    }

                    match config.set_str(key, value) {
                        Ok(()) => {
                            // Windows 上需要显式关闭 Config 对象，确保文件完全释放
                            #[cfg(target_os = "windows")]
                            {
                                drop(config);
                                // 在 CI 环境中可能需要更长的等待时间
                                // 确保文件系统操作完成，特别是重命名操作
                                std::thread::sleep(std::time::Duration::from_millis(150));
                            }
                            return Ok(());
                        }
                        Err(e) => {
                            // Windows 上显式关闭 Config 对象
                            #[cfg(target_os = "windows")]
                            {
                                drop(config);
                            }

                            // 检查是否是锁文件错误（包括 Windows 特定的错误消息）
                            let error_msg = e.message();
                            let is_lock_error = error_msg.contains("could not lock config file")
                                || error_msg.contains("failed to rename lockfile")
                                || error_msg.contains("Access is denied")
                                || error_msg.contains("failed to rename"); // 更通用的匹配

                            // 如果是锁文件错误且还有重试机会，等待后重试
                            if is_lock_error && attempt < MAX_RETRIES - 1 {
                                // Windows 上使用指数退避策略，并增加基础延迟
                                #[cfg(target_os = "windows")]
                                {
                                    // 基础延迟更长，指数增长
                                    let base_delay = RETRY_DELAY_MS * 2; // 400ms 基础延迟
                                    let delay_ms = base_delay * (1 << attempt.min(4)); // 最多 16 倍
                                    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                                }
                                #[cfg(not(target_os = "windows"))]
                                {
                                    std::thread::sleep(std::time::Duration::from_millis(
                                        RETRY_DELAY_MS,
                                    ));
                                }
                                continue;
                            }

                            // 其他错误或重试次数用尽，返回错误
                            return Err(color_eyre::eyre::eyre!(
                                "Failed to set Git config {}={} after {} attempts: {}",
                                key,
                                value,
                                attempt + 1,
                                error_msg
                            ));
                        }
                    }
                }
                Err(e) => {
                    // 如果配置文件不存在，创建一个新的
                    if e.code() == git2::ErrorCode::NotFound {
                        // 创建空配置文件（使用规范化后的路径）
                        std::fs::write(&self.config_path, "")
                            .wrap_err("Failed to create config file")?;
                        // 重试
                        if attempt < MAX_RETRIES - 1 {
                            std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS));
                            continue;
                        }
                    }
                    return Err(color_eyre::eyre::eyre!(
                        "Failed to open Git config file {}: {}",
                        self.config_path.display(),
                        e.message()
                    ));
                }
            }
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
        // 注意：这里使用 dirs::home_dir() 而不是 test_home_dir()，
        // 因为此函数的目的是从真实的全局 Git 配置复制到测试隔离的配置中。
        // 即使测试环境设置了 HOME 环境变量，我们也需要访问真实的系统主目录。
        let global_config = dirs::home_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get home directory"))?
            .join(".gitconfig");

        if global_config.exists() {
            // 复制全局配置到临时文件（使用规范化后的路径）
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
    /// 3. 使用 git2 API 验证配置已设置
    ///
    /// ## 预期结果
    /// - 配置项设置成功
    /// - git2 API 能够读取设置的配置值
    #[test]
    fn test_git_config_guard_set_return_ok() -> Result<()> {
        let guard = GitConfigGuard::new()?;

        guard.set("user.name", "Test User")?;
        guard.set("user.email", "test@example.com")?;

        // 验证配置已设置

        // 使用环境变量中的路径（git2 会读取 GIT_CONFIG 环境变量）
        // 或者直接使用 config_path
        let config_path = guard.config_path();

        // 确保文件存在
        if !config_path.exists() {
            return Err(color_eyre::eyre::eyre!(
                "Config file does not exist: {}",
                config_path.display()
            ));
        }

        // Windows 上，git2 的 Config::open() 可能需要使用环境变量
        // 或者我们需要确保使用正确的路径格式
        #[cfg(target_os = "windows")]
        {
            // 在 Windows 上，优先使用环境变量中的路径
            if let Ok(env_path) = std::env::var("GIT_CONFIG") {
                if let Ok(config) = Config::open(std::path::Path::new(&env_path)) {
                    if let Ok(name) = config.get_string("user.name") {
                        assert_eq!(name, "Test User");
                        return Ok(());
                    }
                }
            }
        }

        // 打开配置文件（使用绝对路径）
        // canonicalize 需要 &Path，而 config_path 是 &PathBuf，需要解引用
        let abs_path =
            std::fs::canonicalize(config_path.as_path()).unwrap_or_else(|_| config_path.clone());
        let config = Config::open(&abs_path)?;

        // 读取配置值
        let name = config.get_string("user.name")?;
        assert_eq!(name, "Test User");

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

        // 保存 guard 设置的路径，用于验证 drop 后是否被清理
        let guard_set_path = {
            let guard = GitConfigGuard::new()?;
            // 验证GIT_CONFIG已设置
            let guard_path = std::env::var("GIT_CONFIG")
                .map_err(|e| color_eyre::eyre::eyre!("GIT_CONFIG should be set: {}", e))?;

            let config_path_str = guard.config_path().to_string_lossy().to_string();

            // 在 Windows 上，路径格式可能不同（短路径 vs 长路径），需要规范化比较
            // 移除 \\?\ 前缀（如果存在）并统一路径分隔符以便比较
            #[cfg(target_os = "windows")]
            {
                let normalize = |p: &str| -> String {
                    p.trim_start_matches("\\\\?\\").replace('\\', "/").to_lowercase()
                };
                let normalized_guard_path = normalize(&guard_path);
                let normalized_config_path = normalize(&config_path_str);
                assert_eq!(
                    normalized_guard_path, normalized_config_path,
                    "GIT_CONFIG should point to guard's config path (guard_path: {}, config_path: {})",
                    guard_path, config_path_str
                );
            }
            #[cfg(not(target_os = "windows"))]
            {
                assert_eq!(
                    guard_path, config_path_str,
                    "GIT_CONFIG should point to guard's config path"
                );
            }
            config_path_str
        };

        // 等待 Drop 实现完成（所有平台都可能需要短暂延迟）
        std::thread::sleep(std::time::Duration::from_millis(10));

        // 验证GIT_CONFIG已恢复
        match original_git_config {
            Some(ref val) => {
                let current = std::env::var("GIT_CONFIG")
                    .map_err(|e| color_eyre::eyre::eyre!("GIT_CONFIG should exist: {}", e))?;

                // 验证 guard 设置的路径已经被清理（不应该再是 guard 设置的路径）
                assert_ne!(
                    current, guard_set_path,
                    "GIT_CONFIG should not point to guard's config path after drop"
                );

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
            None => {
                // 所有平台都使用重试机制，因为环境变量清理可能有延迟
                // 特别是在并行测试环境中，其他测试可能也在设置 GIT_CONFIG
                let mut retries = 0;
                let max_retries = 10;
                while retries < max_retries {
                    match std::env::var("GIT_CONFIG") {
                        Ok(current) => {
                            // 如果环境变量仍然存在，检查是否是 guard 设置的路径
                            // 如果是，说明 drop 没有正确执行
                            if current == guard_set_path {
                                // Guard 设置的路径仍然存在，说明 drop 没有正确执行
                                if retries < max_retries - 1 {
                                    std::thread::sleep(std::time::Duration::from_millis(10));
                                    retries += 1;
                                    continue;
                                } else {
                                    return Err(color_eyre::eyre::eyre!(
                                        "GIT_CONFIG still points to guard's config path after drop: {}",
                                        current
                                    ));
                                }
                            } else {
                                // 环境变量存在，但不是 guard 设置的路径
                                // 可能是其他测试设置的，这在并行测试环境中是可以接受的
                                // 我们只验证 guard 的路径已经被清理
                                break;
                            }
                        }
                        Err(_) => {
                            // 环境变量已被清理，测试通过
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
