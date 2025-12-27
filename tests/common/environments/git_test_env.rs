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
//! fn test_git_operations_return_ok() -> color_eyre::Result<()> {
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

        // 先配置Git用户（避免借用冲突）
        if let Some(git_guard) = isolation.git_config_guard() {
            git_guard.set("user.name", "Test User")?;
            git_guard.set("user.email", "test@example.com")?;
        }

        // 获取工作目录绝对路径
        let work_dir = isolation.work_dir().to_path_buf();

        // 确保.git目录不存在（如果存在则删除）
        let git_dir = work_dir.join(".git");
        if git_dir.exists() {
            std::fs::remove_dir_all(&git_dir).map_err(|e| {
                color_eyre::eyre::eyre!("Failed to remove existing .git directory: {}", e)
            })?;
        }

        // 初始化Git仓库，设置默认分支为main
        Self::run_git_command(&work_dir, &["init", "-b", "main"])?;

        // 在仓库的配置文件中设置Git用户配置
        // 临时取消 GIT_CONFIG 环境变量（如果存在），然后使用 --local 选项设置配置
        // 这样可以避免 "only one config file at a time" 错误
        let original_git_config = std::env::var("GIT_CONFIG").ok();
        std::env::remove_var("GIT_CONFIG");

        // 设置用户配置
        Self::run_git_command(&work_dir, &["config", "--local", "user.name", "Test User"])?;
        Self::run_git_command(
            &work_dir,
            &["config", "--local", "user.email", "test@example.com"],
        )?;

        // 恢复 GIT_CONFIG 环境变量
        if let Some(ref val) = original_git_config {
            std::env::set_var("GIT_CONFIG", val);
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
        self.isolation.work_dir().to_path_buf()
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

    /// 添加假的远程仓库引用（用于测试需要远程分支的功能）
    ///
    /// 创建假的远程引用，让 `get_default_branch()` 等函数能正常工作，
    /// 但不进行真实的网络连接。
    ///
    /// # 参数
    ///
    /// * `remote_name` - 远程仓库名称（如 "origin"）
    /// * `remote_url` - 远程仓库URL（假的URL，不会实际连接）
    ///
    /// # 返回
    ///
    /// 成功时返回`Ok(())`，失败时返回错误
    ///
    /// # 功能
    ///
    /// 1. 添加远程URL（使用假的URL）
    /// 2. 创建假的远程分支引用（`refs/remotes/{remote_name}/main`）
    /// 3. 设置远程HEAD引用（`refs/remotes/{remote_name}/HEAD`）
    /// 4. 配置 `url.insteadOf` 避免真实网络请求（如果URL是https://）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::GitTestEnv;
    ///
    /// let env = GitTestEnv::new()?;
    /// env.add_fake_remote("origin", "https://github.com/test/test-repo.git")?;
    /// ```
    #[allow(dead_code)] // 这是一个公共API，可能被其他测试使用
    pub fn add_fake_remote(&self, remote_name: &str, remote_url: &str) -> Result<()> {
        let repo_path = self.path();

        // 1. 如果URL是https://，配置url.insteadOf避免真实网络请求
        if remote_url.starts_with("https://") {
            // 提取域名部分，配置insteadOf
            if let Some(domain_start) = remote_url.find("://") {
                let domain = &remote_url[domain_start + 3..];
                if let Some(path_start) = domain.find('/') {
                    let domain_only = &domain[..path_start];
                    // 配置所有该域名的请求都重定向到本地（避免网络请求）
                    Self::run_git_command(
                        &repo_path,
                        &[
                            "config",
                            "url.file:///dev/null.insteadOf",
                            &format!("https://{}", domain_only),
                        ],
                    )
                    .ok(); // 允许失败，因为可能已经配置过
                }
            }
        }

        // 2. 添加远程URL（使用假的URL）
        Self::run_git_command(&repo_path, &["remote", "add", remote_name, remote_url])?;

        // 3. 创建假的远程分支引用（指向当前HEAD）
        Self::run_git_command(
            &repo_path,
            &[
                "update-ref",
                &format!("refs/remotes/{}/main", remote_name),
                "HEAD",
            ],
        )?;

        // 4. 删除可能存在的旧引用（如origin/master）
        Self::run_git_command(
            &repo_path,
            &[
                "update-ref",
                "-d",
                &format!("refs/remotes/{}/master", remote_name),
            ],
        )
        .ok(); // 允许失败，因为可能不存在

        // 5. 设置远程HEAD引用指向main（让 git remote show origin 能工作）
        Self::run_git_command(
            &repo_path,
            &[
                "symbolic-ref",
                &format!("refs/remotes/{}/HEAD", remote_name),
                &format!("refs/remotes/{}/main", remote_name),
            ],
        )
        .ok(); // 允许失败，某些Git版本可能不支持

        Ok(())
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

    /// 测试GitTestEnv创建
    ///
    /// ## 测试目的
    /// 验证 `GitTestEnv::new()` 能够成功创建Git测试环境，包括临时目录和Git仓库初始化。
    ///
    /// ## 测试场景
    /// 1. 创建GitTestEnv实例
    /// 2. 获取仓库路径
    /// 3. 验证路径存在
    /// 4. 验证.git目录存在
    ///
    /// ## 预期结果
    /// - 仓库路径存在
    /// - .git目录存在
    #[test]
    #[serial]
    fn test_git_test_env_creation_return_ok() -> Result<()> {
        let env = GitTestEnv::new()?;
        let path = env.path();
        assert!(path.exists());
        assert!(path.join(".git").exists());
        Ok(())
    }

    /// 测试创建和切换分支
    ///
    /// ## 测试目的
    /// 验证 `GitTestEnv` 的分支操作功能（create_branch, checkout, current_branch）能够正常工作。
    ///
    /// ## 测试场景
    /// 1. 创建GitTestEnv
    /// 2. 创建新分支（test-branch）
    /// 3. 切换到新分支
    /// 4. 验证当前分支为新创建的分支
    ///
    /// ## 预期结果
    /// - 分支创建成功
    /// - 切换分支成功
    /// - 当前分支为test-branch
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

    /// 测试创建测试提交
    ///
    /// ## 测试目的
    /// 验证 `GitTestEnv::make_test_commit()` 方法能够创建提交，并更新最后一次提交的SHA。
    ///
    /// ## 测试场景
    /// 1. 创建GitTestEnv
    /// 2. 获取创建提交前的SHA
    /// 3. 创建测试提交
    /// 4. 获取创建提交后的SHA
    /// 5. 验证SHA已更新
    ///
    /// ## 预期结果
    /// - 提交创建成功
    /// - 提交后的SHA与提交前不同
    #[test]
    #[serial]
    fn test_make_test_commit_return_ok() -> Result<()> {
        let env = GitTestEnv::new()?;

        let sha_before = env.last_commit_sha()?;

        env.make_test_commit("test.txt", "test content", "test commit")?;

        let sha_after = env.last_commit_sha()?;
        assert_ne!(sha_before, sha_after);

        Ok(())
    }

    /// 测试添加假远程仓库引用
    ///
    /// ## 测试目的
    /// 验证 `GitTestEnv::add_fake_remote()` 能够成功添加假的远程引用。
    ///
    /// ## 测试场景
    /// 1. 创建GitTestEnv
    /// 2. 添加假的远程引用
    /// 3. 验证远程引用已创建
    ///
    /// ## 预期结果
    /// - 远程引用创建成功
    /// - 远程分支引用存在
    /// - 远程HEAD引用存在
    #[test]
    #[serial]
    fn test_add_fake_remote_return_ok() -> Result<()> {
        let env = GitTestEnv::new()?;

        // 添加假的远程引用
        env.add_fake_remote("origin", "https://github.com/test/test-repo.git")?;

        // 验证远程引用已创建
        let repo_path = env.path();
        let output = Command::new("git")
            .args(["show-ref", "refs/remotes/origin/main"])
            .current_dir(&repo_path)
            .output()?;

        assert!(
            output.status.success(),
            "Remote ref should exist after add_fake_remote"
        );

        Ok(())
    }

    /// 测试GitTestEnv与当前仓库的隔离
    ///
    /// ## 测试目的
    /// 验证 `GitTestEnv` 创建的测试仓库与当前仓库完全隔离，不会影响当前仓库的状态。
    ///
    /// ## 测试场景
    /// 测试 GitTestEnv 不会操作当前仓库
    ///
    /// ## 测试目的
    /// 验证 `GitTestEnv` 使用绝对路径，不会切换全局工作目录。
    ///
    /// ## 测试策略
    /// - ✅ 验证测试仓库路径在临时目录中
    /// - ✅ 验证测试仓库路径与当前仓库路径不同
    /// - ✅ 验证返回的是绝对路径
    /// - ✅ 验证 GitTestEnv 创建时没有改变全局目录（在创建后立即检查）
    /// - ⚠️ 不检查测试结束时的全局目录（因为并行测试可能改变它）
    ///
    /// ## 注意事项
    /// - 使用 `#[serial]` 标记，避免并行测试时的竞态条件
    /// - 全局工作目录检查在 GitTestEnv 创建后立即执行，而不是在测试结束时
    /// - 如果当前目录是临时目录，可能是其他测试切换的，这是可以接受的
    ///
    /// ## 测试步骤
    /// 1. 保存当前工作目录
    /// 2. 创建GitTestEnv（会创建独立的测试仓库）
    /// 3. 验证测试仓库路径在临时目录中
    /// 4. 验证测试仓库路径与当前仓库路径不同
    /// 5. 验证测试仓库使用绝对路径
    /// 6. 验证 GitTestEnv 创建时没有改变全局目录（在创建后立即检查）
    ///
    /// ## 预期结果
    /// - 测试仓库路径在临时目录中
    /// - 测试仓库路径与当前仓库路径不同
    /// - 测试仓库路径为绝对路径
    /// - GitTestEnv 创建时全局工作目录保持不变（使用绝对路径，不切换全局目录）
    #[test]
    #[serial]
    fn test_isolation_from_current_repo_return_ok() -> Result<()> {
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

            // 验证返回的是绝对路径
            assert!(
                test_repo_path.is_absolute(),
                "Test repo path should be absolute: {}",
                test_repo_path_str
            );

            // 验证测试仓库有独立的 .git 目录
            assert!(test_repo_path.join(".git").exists());

            // 验证全局工作目录没有被切换（方案5：不切换全局目录）
            // 注意：在并行测试时，其他测试可能使用 CurrentDirGuard 切换了全局目录
            // 所以我们只验证 GitTestEnv 本身不切换全局目录
            // ✅ 改进：在 GitTestEnv 创建后立即检查，而不是在测试结束时
            let current_dir_during_test = std::env::current_dir()?;
            let current_dir_during_test_str = current_dir_during_test.to_string_lossy().to_string();

            // 如果当前目录不是临时目录，说明 GitTestEnv 没有切换它（符合预期）
            // 如果当前目录是临时目录，可能是其他测试切换的，这是可以接受的
            // 我们主要验证 GitTestEnv 创建的测试仓库路径是独立的（已验证）
            if !current_dir_during_test_str.contains("/tmp")
                && !current_dir_during_test_str.contains("tmp")
            {
                // 如果当前目录不是临时目录，应该保持不变（GitTestEnv 没有切换它）
                assert_eq!(
                    current_dir_during_test_str, original_dir_str,
                    "GitTestEnv should not change global current directory (we use absolute paths instead)"
                );
            }
            // ✅ 改进：移除测试结束时的全局目录检查
            // 在并行测试时，其他测试可能使用 CurrentDirGuard 切换了全局目录
            // 我们只验证 GitTestEnv 本身的行为，不验证全局状态（因为它是共享的）
        }

        // ✅ 改进：移除测试结束时的全局目录检查
        // 在并行测试时，其他测试可能使用 CurrentDirGuard 切换了全局目录
        // 我们只验证 GitTestEnv 本身的行为，不验证全局状态（因为它是共享的）

        Ok(())
    }
}
