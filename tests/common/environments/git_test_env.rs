//! 统一Git测试环境
//!
//! 基于 TestIsolation 的 Git 测试环境，提供完全隔离的 Git 仓库操作。
//!
//! # 使用示例
//!
//! ```rust
//! use tests::common::environments::GitTestEnv;
//!
//! #[test]
//! fn test_git_operations() -> color_eyre::Result<()> {
//!     let env = GitTestEnv::new()?;
//!
//!     env.create_branch("feature/test")?;
//!     env.checkout("feature/test")?;
//!     env.make_test_commit("test.txt", "content", "test commit")?;
//!
//!     Ok(())
//! }
//! ```

use color_eyre::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::common::isolation::TestIsolation;

/// 统一的Git测试环境
///
/// 基于`TestIsolation`构建，提供完全隔离的Git测试环境，包括：
/// - 独立的工作目录
/// - 隔离的环境变量
/// - 独立的Git配置
/// - 自动初始化的Git仓库
///
/// # 功能特性
///
/// - ✅ 完全隔离的测试环境
/// - ✅ 自动初始化Git仓库
/// - ✅ 自动配置测试用户
/// - ✅ 自动创建初始提交
/// - ✅ RAII模式自动清理
pub struct GitTestEnv {
    /// 测试隔离管理器
    isolation: TestIsolation,
}

impl GitTestEnv {
    /// 创建新的Git测试环境
    ///
    /// 自动创建隔离环境并初始化Git仓库，包括：
    /// - 创建临时目录并切换工作目录
    /// - 初始化Git配置隔离
    /// - 初始化Git仓库（默认分支为main）
    /// - 配置测试用户（Test User <test@example.com>）
    /// - 创建初始提交
    ///
    /// # 返回
    ///
    /// 成功时返回`GitTestEnv`实例，失败时返回错误
    ///
    /// # 错误
    ///
    /// - 无法创建隔离环境
    /// - 无法初始化Git仓库
    /// - 无法配置Git用户
    /// - 无法创建初始提交
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::GitTestEnv;
    ///
    /// let env = GitTestEnv::new()?;
    /// ```
    pub fn new() -> Result<Self> {
        // 创建隔离环境，启用Git配置隔离
        let mut isolation = TestIsolation::new()?.with_git_config()?;

        let work_dir = isolation.work_dir();

        // 确保.git目录不存在（如果存在则删除）
        let git_dir = work_dir.join(".git");
        if git_dir.exists() {
            std::fs::remove_dir_all(&git_dir).map_err(|e| {
                color_eyre::eyre::eyre!("Failed to remove existing .git directory: {}", e)
            })?;
        }

        // 初始化Git仓库，设置默认分支为main
        Self::run_git_command(&work_dir, &["init", "-b", "main"])?;

        // 配置测试用户（使用Git配置守卫）
        if let Some(git_guard) = isolation.git_config_guard() {
            git_guard.set("user.name", "Test User")?;
            git_guard.set("user.email", "test@example.com")?;
        }

        // 创建初始提交
        std::fs::write(work_dir.join("README.md"), "# Test Repository\n")?;
        Self::run_git_command(&work_dir, &["add", "."])?;
        Self::run_git_command(&work_dir, &["commit", "-m", "Initial commit"])?;

        Ok(Self { isolation })
    }

    /// 获取仓库路径
    ///
    /// # 返回
    ///
    /// 返回Git仓库的路径
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::GitTestEnv;
    ///
    /// let env = GitTestEnv::new()?;
    /// let repo_path = env.path();
    /// ```
    pub fn path(&self) -> PathBuf {
        self.isolation.work_dir()
    }

    /// 创建新分支
    ///
    /// # 参数
    ///
    /// * `branch_name` - 分支名
    ///
    /// # 返回
    ///
    /// 成功时返回`Ok(())`，失败时返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::GitTestEnv;
    ///
    /// let env = GitTestEnv::new()?;
    /// env.create_branch("feature/test")?;
    /// ```
    pub fn create_branch(&self, branch_name: &str) -> Result<()> {
        Self::run_git_command(&self.path(), &["branch", branch_name])
    }

    /// 切换分支
    ///
    /// # 参数
    ///
    /// * `branch_name` - 分支名
    ///
    /// # 返回
    ///
    /// 成功时返回`Ok(())`，失败时返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::GitTestEnv;
    ///
    /// let env = GitTestEnv::new()?;
    /// env.checkout("feature/test")?;
    /// ```
    pub fn checkout(&self, branch_name: &str) -> Result<()> {
        Self::run_git_command(&self.path(), &["checkout", branch_name])
    }

    /// 创建并切换到新分支
    ///
    /// # 参数
    ///
    /// * `branch_name` - 分支名
    ///
    /// # 返回
    ///
    /// 成功时返回`Ok(())`，失败时返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::GitTestEnv;
    ///
    /// let env = GitTestEnv::new()?;
    /// env.checkout_new_branch("feature/test")?;
    /// ```
    pub fn checkout_new_branch(&self, branch_name: &str) -> Result<()> {
        Self::run_git_command(&self.path(), &["checkout", "-b", branch_name])
    }

    /// 创建测试文件
    ///
    /// # 参数
    ///
    /// * `filename` - 文件名
    /// * `content` - 文件内容
    ///
    /// # 返回
    ///
    /// 成功时返回`Ok(())`，失败时返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::GitTestEnv;
    ///
    /// let env = GitTestEnv::new()?;
    /// env.create_file("test.txt", "test content")?;
    /// ```
    pub fn create_file(&self, filename: &str, content: &str) -> Result<()> {
        let file_path = self.path().join(filename);
        std::fs::write(file_path, content)?;
        Ok(())
    }

    /// 添加并提交更改
    ///
    /// # 参数
    ///
    /// * `message` - 提交消息
    ///
    /// # 返回
    ///
    /// 成功时返回`Ok(())`，失败时返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::GitTestEnv;
    ///
    /// let env = GitTestEnv::new()?;
    /// env.create_file("test.txt", "content")?;
    /// env.add_and_commit("Add test file")?;
    /// ```
    pub fn add_and_commit(&self, message: &str) -> Result<()> {
        Self::run_git_command(&self.path(), &["add", "."])?;
        Self::run_git_command(&self.path(), &["commit", "-m", message])
    }

    /// 创建测试提交
    ///
    /// 创建文件并提交的便捷方法。
    ///
    /// # 参数
    ///
    /// * `filename` - 文件名
    /// * `content` - 文件内容
    /// * `message` - 提交消息
    ///
    /// # 返回
    ///
    /// 成功时返回`Ok(())`，失败时返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::GitTestEnv;
    ///
    /// let env = GitTestEnv::new()?;
    /// env.make_test_commit("test.txt", "content", "test commit")?;
    /// ```
    pub fn make_test_commit(&self, filename: &str, content: &str, message: &str) -> Result<()> {
        self.create_file(filename, content)?;
        self.add_and_commit(message)
    }

    /// 获取当前分支名
    ///
    /// # 返回
    ///
    /// 成功时返回当前分支名，失败时返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::GitTestEnv;
    ///
    /// let env = GitTestEnv::new()?;
    /// let branch = env.current_branch()?;
    /// assert_eq!(branch, "main");
    /// ```
    pub fn current_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(&self.path())
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!(
                "Failed to get current branch: {}",
                error
            ));
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// 获取最后一次提交的SHA
    ///
    /// # 返回
    ///
    /// 成功时返回最后一次提交的SHA，失败时返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::GitTestEnv;
    ///
    /// let env = GitTestEnv::new()?;
    /// let sha = env.last_commit_sha()?;
    /// ```
    pub fn last_commit_sha(&self) -> Result<String> {
        let output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(&self.path())
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!(
                "Failed to get commit SHA: {}",
                error
            ));
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// 获取环境变量守卫的可变引用（用于设置环境变量）
    ///
    /// # 返回
    ///
    /// 返回环境变量守卫的可变引用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::GitTestEnv;
    ///
    /// let mut env = GitTestEnv::new()?;
    /// env.env_guard().set("HOME", "/tmp/test");
    /// ```
    #[allow(dead_code)]
    pub fn env_guard(&mut self) -> &mut crate::common::guards::EnvGuard {
        self.isolation.env_guard()
    }

    /// 运行Git命令
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库路径
    /// * `args` - Git命令参数
    ///
    /// # 返回
    ///
    /// 成功时返回`Ok(())`，失败时返回错误
    fn run_git_command(repo_path: &Path, args: &[&str]) -> Result<()> {
        let output = Command::new("git").args(args).current_dir(repo_path).output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!(
                "Git command failed: git {}\nError: {}",
                args.join(" "),
                error
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_git_test_env_creation() -> Result<()> {
        let env = GitTestEnv::new()?;
        let path = env.path();
        assert!(path.exists());
        assert!(path.join(".git").exists());
        Ok(())
    }

    #[test]
    #[serial]
    fn test_create_and_checkout_branch() -> Result<()> {
        let env = GitTestEnv::new()?;

        env.create_branch("test-branch")?;
        env.checkout("test-branch")?;

        let current = env.current_branch()?;
        assert_eq!(current, "test-branch");

        Ok(())
    }

    #[test]
    #[serial]
    fn test_make_test_commit() -> Result<()> {
        let env = GitTestEnv::new()?;

        let sha_before = env.last_commit_sha()?;

        env.make_test_commit("test.txt", "test content", "test commit")?;

        let sha_after = env.last_commit_sha()?;
        assert_ne!(sha_before, sha_after);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_isolation_from_current_repo() -> Result<()> {
        // 验证 GitTestEnv 不会操作当前仓库
        let original_dir = std::env::current_dir()?;
        let original_dir_str = original_dir.to_string_lossy().to_string();

        {
            let env = GitTestEnv::new()?;
            let test_repo_path = env.path();
            let test_repo_path_str = test_repo_path.to_string_lossy().to_string();

            // 验证测试仓库路径在临时目录中，不在当前仓库
            assert!(
                test_repo_path_str.contains("/tmp") || test_repo_path_str.contains("tmp"),
                "Test repo should be in temp directory, got: {}",
                test_repo_path_str
            );
            assert_ne!(
                test_repo_path_str, original_dir_str,
                "Test repo path should not be the current repo path"
            );

            // 验证当前工作目录已切换到临时目录
            let current_dir = std::env::current_dir()?;
            let current_dir_str = current_dir.to_string_lossy().to_string();
            assert_eq!(
                current_dir_str, test_repo_path_str,
                "Current directory should be the test repo directory"
            );

            // 验证测试仓库有独立的 .git 目录
            assert!(test_repo_path.join(".git").exists());
        }

        // 验证工作目录已恢复
        let restored_dir = std::env::current_dir()?;
        let restored_dir_str = restored_dir.to_string_lossy().to_string();
        assert_eq!(
            restored_dir_str, original_dir_str,
            "Current directory should be restored after GitTestEnv drop"
        );

        Ok(())
    }
}
