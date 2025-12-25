//! 统一测试隔离管理器
//!
//! 提供完全隔离的测试环境，包括工作目录、环境变量、Git配置和Mock服务器。
//!
//! # 使用示例
//!
//! ```rust
//! use tests::common::TestIsolation;
//!
//! #[test]
//! fn test_with_full_isolation() -> color_eyre::Result<()> {
//!     let isolation = TestIsolation::new()?
//!         .with_git_config()?
//!         .with_mock_server()?;
//!
//!     // 测试代码在完全隔离的环境中运行
//!     // ...
//!
//!     Ok(())
//!     // isolation在此自动清理
//! }
//! ```

use color_eyre::Result;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::common::guards::{EnvGuard, GitConfigGuard};
use crate::common::http_helpers::MockServer;

/// 统一测试隔离管理器
///
/// 提供完全隔离的测试环境，包括：
/// - 独立的工作目录（使用绝对路径，不切换全局工作目录）
/// - 隔离的环境变量（EnvGuard）
/// - 独立的Git配置（GitConfigGuard，可选）
/// - 独立的Mock服务器（MockServer，可选）
///
/// # 功能特性
///
/// - ✅ RAII模式自动清理
/// - ✅ 支持嵌套隔离
/// - ✅ 线程安全（不切换全局工作目录，避免竞态条件）
/// - ✅ 可配置的隔离级别
/// - ✅ 使用绝对路径，完全避免工作目录竞态条件
pub struct TestIsolation {
    /// 临时目录（保持引用以确保目录不被删除）
    _temp_dir: TempDir,
    /// 工作目录绝对路径（从 temp_dir.path() 获取，不切换全局目录）
    work_dir: PathBuf,
    /// 环境变量守卫
    env_guard: EnvGuard,
    /// Git配置守卫（可选）
    git_config_guard: Option<GitConfigGuard>,
    /// Mock服务器（可选）
    mock_server: Option<MockServer>,
}

impl TestIsolation {
    /// 创建基础隔离环境
    ///
    /// 创建临时目录（使用绝对路径），同时初始化环境变量隔离。
    /// **注意**: 不切换全局工作目录，所有操作使用绝对路径，避免并行测试时的竞态条件。
    ///
    /// # 返回
    ///
    /// 成功时返回`TestIsolation`实例，失败时返回错误
    ///
    /// # 错误
    ///
    /// - 无法创建临时目录
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::TestIsolation;
    ///
    /// let isolation = TestIsolation::new()?;
    /// let work_dir = isolation.work_dir();  // 获取绝对路径
    /// ```
    pub fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create temp directory: {}", e))?;

        // 获取绝对路径，不切换全局工作目录（避免并行测试竞态条件）
        let work_dir = temp_dir.path().to_path_buf();
        let env_guard = EnvGuard::new();

        Ok(Self {
            _temp_dir: temp_dir,
            work_dir,
            env_guard,
            git_config_guard: None,
            mock_server: None,
        })
    }

    /// 启用Git配置隔离
    ///
    /// 创建独立的Git配置环境，测试结束后自动恢复。
    ///
    /// # 返回
    ///
    /// 返回`Self`以支持链式调用
    ///
    /// # 错误
    ///
    /// - 无法创建Git配置守卫
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::TestIsolation;
    ///
    /// let isolation = TestIsolation::new()?
    ///     .with_git_config()?;
    /// ```
    pub fn with_git_config(mut self) -> Result<Self> {
        let git_config_guard = GitConfigGuard::new()?;
        self.git_config_guard = Some(git_config_guard);
        Ok(self)
    }

    /// 启用Mock服务器
    ///
    /// 创建独立的Mock服务器，测试结束后自动清理。
    ///
    /// # 返回
    ///
    /// 返回`Self`以支持链式调用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::TestIsolation;
    ///
    /// let isolation = TestIsolation::new()?
    ///     .with_mock_server()?;
    /// ```
    pub fn with_mock_server(mut self) -> Result<Self> {
        let mock_server = MockServer::new();
        self.mock_server = Some(mock_server);
        Ok(self)
    }

    /// 获取工作目录路径（绝对路径）
    ///
    /// # 返回
    ///
    /// 返回临时工作目录的绝对路径
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::TestIsolation;
    ///
    /// let isolation = TestIsolation::new()?;
    /// let work_dir = isolation.work_dir();  // 绝对路径
    /// let file_path = work_dir.join("test.txt");
    /// ```
    pub fn work_dir(&self) -> &std::path::Path {
        // 返回绝对路径，不依赖全局工作目录
        &self.work_dir
    }

    /// 获取环境变量守卫的可变引用
    ///
    /// 用于在测试中设置环境变量。
    ///
    /// # 返回
    ///
    /// 返回环境变量守卫的可变引用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::TestIsolation;
    ///
    /// let mut isolation = TestIsolation::new()?;
    /// isolation.env_guard().set("HOME", "/tmp/test");
    /// ```
    pub fn env_guard(&mut self) -> &mut EnvGuard {
        &mut self.env_guard
    }

    /// 获取Git配置守卫的可变引用
    ///
    /// 用于在测试中设置Git配置。
    ///
    /// # 返回
    ///
    /// 返回Git配置守卫的可变引用，如果未启用则返回`None`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::TestIsolation;
    ///
    /// let mut isolation = TestIsolation::new()?.with_git_config()?;
    /// isolation.git_config_guard().unwrap().set("user.name", "Test User")?;
    /// ```
    pub fn git_config_guard(&mut self) -> Option<&mut GitConfigGuard> {
        self.git_config_guard.as_mut()
    }

    /// 获取Mock服务器引用
    ///
    /// 用于在测试中设置Mock端点。
    ///
    /// # 返回
    ///
    /// 返回Mock服务器的引用，如果未启用则返回`None`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::TestIsolation;
    ///
    /// let isolation = TestIsolation::new()?.with_mock_server()?;
    /// let mock_server = isolation.mock_server().unwrap();
    /// mock_server.setup_github_base_url();
    /// ```
    pub fn mock_server(&self) -> Option<&MockServer> {
        self.mock_server.as_ref()
    }

    /// 获取Mock服务器的可变引用
    ///
    /// 用于在测试中设置Mock端点。
    ///
    /// # 返回
    ///
    /// 返回Mock服务器的可变引用，如果未启用则返回`None`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::TestIsolation;
    ///
    /// let mut isolation = TestIsolation::new()?.with_mock_server()?;
    /// let mock_server = isolation.mock_server_mut().unwrap();
    /// mock_server.setup_github_base_url();
    /// ```
    #[allow(unused)]
    pub fn mock_server_mut(&mut self) -> Option<&mut MockServer> {
        self.mock_server.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_isolation_basic() -> Result<()> {
        let isolation = TestIsolation::new()?;
        let work_dir = isolation.work_dir();

        // 验证工作目录存在
        assert!(work_dir.exists());
        assert!(work_dir.is_dir());

        // 验证返回的是绝对路径
        assert!(work_dir.is_absolute());

        // 验证路径来自tempfile临时目录
        assert!(work_dir.to_string_lossy().contains("tmp") ||
                work_dir.to_string_lossy().contains("temp"));

        Ok(())
    }

    #[test]
    fn test_test_isolation_with_git_config() -> Result<()> {
        let mut isolation = TestIsolation::new()?.with_git_config()?;

        // 验证Git配置守卫可用
        let git_guard = isolation.git_config_guard().unwrap();
        git_guard.set("user.name", "Test User")?;
        git_guard.set("user.email", "test@example.com")?;

        Ok(())
    }

    #[test]
    fn test_test_isolation_with_mock_server() -> Result<()> {
        let isolation = TestIsolation::new()?.with_mock_server()?;

        // 验证Mock服务器可用
        let mock_server = isolation.mock_server().unwrap();
        assert!(!mock_server.base_url.is_empty());

        Ok(())
    }

    #[test]
    fn test_test_isolation_env_guard() -> Result<()> {
        let mut isolation = TestIsolation::new()?;

        // 设置环境变量
        isolation.env_guard().set("TEST_VAR", "test_value");

        // 验证环境变量已设置
        assert_eq!(std::env::var("TEST_VAR").unwrap(), "test_value");

        // Drop时自动恢复
        Ok(())
    }
}
