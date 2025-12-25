//! 统一CLI测试环境
//!
//! 基于 TestIsolation 的 CLI 测试环境，提供完全隔离的 CLI 测试环境。
//!
//! # 使用示例
//!
//! ```rust
//! use tests::common::environments::CliTestEnv;
//!
//! #[test]
//! fn test_cli_command_return_result() -> color_eyre::Result<()> {
//!     let env = CliTestEnv::new()?;
//!     env.init_git_repo()?;
//!     env.create_file("test.txt", "content")?;
//!
//!     Ok(())
//! }
//! ```

use color_eyre::Result;
use std::fs;
use std::path::PathBuf;

use crate::common::isolation::TestIsolation;

/// 统一的CLI测试环境
///
/// 基于`TestIsolation`构建，提供完全隔离的CLI测试环境，包括：
/// - 独立的工作目录
/// - 隔离的环境变量
/// - 独立的Git配置（可选）
/// - 便捷的文件和配置管理
///
/// # 功能特性
///
/// - ✅ 完全隔离的测试环境
/// - ✅ 支持Git仓库初始化
/// - ✅ 便捷的文件和配置管理
/// - ✅ RAII模式自动清理
pub struct CliTestEnv {
    /// 测试隔离管理器
    isolation: TestIsolation,
}

impl CliTestEnv {
    /// 创建新的CLI测试环境
    ///
    /// 自动创建隔离环境，包括临时目录和环境变量隔离。
    ///
    /// # 返回
    ///
    /// 成功时返回`CliTestEnv`实例，失败时返回错误
    ///
    /// # 错误
    ///
    /// - 无法创建隔离环境
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::CliTestEnv;
    ///
    /// let env = CliTestEnv::new()?;
    /// ```
    pub fn new() -> Result<Self> {
        let isolation = TestIsolation::new()?;
        Ok(Self { isolation })
    }

    /// 初始化Git仓库
    ///
    /// 在当前工作目录初始化Git仓库，配置测试用户，并添加远程origin。
    ///
    /// # 返回
    ///
    /// 返回`&Self`以支持链式调用
    ///
    /// # 错误
    ///
    /// - 无法初始化Git仓库
    /// - 无法配置Git用户
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::CliTestEnv;
    ///
    /// let env = CliTestEnv::new()?;
    /// env.init_git_repo()?;
    /// ```
    pub fn init_git_repo(&self) -> Result<&Self> {
        let work_dir = self.isolation.work_dir();

        // 确保.git目录不存在（如果存在则删除）
        let git_dir = work_dir.join(".git");
        if git_dir.exists() {
            std::fs::remove_dir_all(&git_dir).map_err(|e| {
                color_eyre::eyre::eyre!("Failed to remove existing .git directory: {}", e)
            })?;
        }

        std::process::Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(&work_dir)
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to init git repo: {}", e))?;

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&work_dir)
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to set git user name: {}", e))?;

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&work_dir)
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to set git user email: {}", e))?;

        // 添加remote origin（用于测试需要remote的功能）
        std::process::Command::new("git")
            .args([
                "remote",
                "add",
                "origin",
                "https://github.com/test/test-repo.git",
            ])
            .current_dir(&work_dir)
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to add remote origin: {}", e))?;

        // 设置远程HEAD引用（模拟远程默认分支为main）
        // 这样get_default_branch()就能正确工作
        std::process::Command::new("git")
            .args([
                "symbolic-ref",
                "refs/remotes/origin/HEAD",
                "refs/remotes/origin/main",
            ])
            .current_dir(&work_dir)
            .output()
            .ok(); // 允许失败，因为可能remote branch还不存在

        Ok(self)
    }

    /// 创建文件
    ///
    /// # 参数
    ///
    /// * `path` - 文件路径（相对于工作目录）
    /// * `content` - 文件内容
    ///
    /// # 返回
    ///
    /// 返回`&Self`以支持链式调用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::CliTestEnv;
    ///
    /// let env = CliTestEnv::new()?;
    /// env.create_file("test.txt", "content")?;
    /// ```
    pub fn create_file(&self, path: &str, content: &str) -> Result<&Self> {
        let work_dir = self.isolation.work_dir();
        let full_path = work_dir.join(path);

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to create parent directory: {}", e))?;
        }

        fs::write(&full_path, content)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to write file: {}", e))?;

        Ok(self)
    }

    /// 创建Git提交
    ///
    /// # 参数
    ///
    /// * `message` - 提交消息
    ///
    /// # 返回
    ///
    /// 返回`&Self`以支持链式调用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::CliTestEnv;
    ///
    /// let env = CliTestEnv::new()?;
    /// env.init_git_repo()?;
    /// env.create_file("test.txt", "content")?;
    /// env.create_commit("Initial commit")?;
    /// ```
    pub fn create_commit(&self, message: &str) -> Result<&Self> {
        let work_dir = self.isolation.work_dir();

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&work_dir)
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to add files: {}", e))?;

        std::process::Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&work_dir)
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to commit: {}", e))?;

        Ok(self)
    }

    /// 创建配置文件
    ///
    /// 在工作目录的`.workflow`目录下创建`workflow.toml`配置文件。
    ///
    /// # 参数
    ///
    /// * `content` - 配置文件内容
    ///
    /// # 返回
    ///
    /// 返回`&Self`以支持链式调用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::CliTestEnv;
    ///
    /// let env = CliTestEnv::new()?;
    /// env.create_config(r#"[jira]
    /// url = "https://test.atlassian.net"
    /// "#)?;
    /// ```
    pub fn create_config(&self, content: &str) -> Result<&Self> {
        let work_dir = self.isolation.work_dir();
        let config_dir = work_dir.join(".workflow");

        fs::create_dir_all(&config_dir)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create config directory: {}", e))?;

        let config_file = config_dir.join("workflow.toml");
        fs::write(&config_file, content)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to write config file: {}", e))?;

        Ok(self)
    }

    /// 获取临时目录路径
    ///
    /// # 返回
    ///
    /// 返回工作目录的路径
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::CliTestEnv;
    ///
    /// let env = CliTestEnv::new()?;
    /// let path = env.path();
    /// ```
    pub fn path(&self) -> PathBuf {
        self.isolation.work_dir().to_path_buf()
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
    /// use tests::common::environments::CliTestEnv;
    ///
    /// let mut env = CliTestEnv::new()?;
    /// env.env_guard().set("HOME", "/tmp/test");
    /// ```
    #[allow(dead_code)]
    pub fn env_guard(&mut self) -> &mut crate::common::guards::EnvGuard {
        self.isolation.env_guard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    /// 测试CliTestEnv创建
    ///
    /// ## 测试目的
    /// 验证 `CliTestEnv::new()` 能够成功创建CLI测试环境，包括临时目录和环境变量隔离。
    ///
    /// ## 测试场景
    /// 1. 创建CliTestEnv实例
    /// 2. 获取路径
    /// 3. 验证路径存在且为目录
    ///
    /// ## 预期结果
    /// - 路径存在
    /// - 路径为目录
    #[test]
    fn test_cli_test_env_creation_return_result() -> Result<()> {
        let env = CliTestEnv::new()?;
        assert!(env.path().exists());
        assert!(env.path().is_dir());
        Ok(())
    }

    /// 测试初始化Git仓库
    ///
    /// ## 测试目的
    /// 验证 `CliTestEnv::init_git_repo()` 方法能够成功初始化Git仓库，包括配置测试用户和添加远程origin。
    ///
    /// ## 测试场景
    /// 1. 创建CliTestEnv
    /// 2. 初始化Git仓库
    /// 3. 验证.git目录存在
    ///
    /// ## 预期结果
    /// - Git仓库初始化成功
    /// - .git目录存在
    #[test]
    #[serial]
    fn test_init_git_repo_return_result() -> Result<()> {
        let env = CliTestEnv::new()?;
        env.init_git_repo()?;

        assert!(env.path().join(".git").exists());
        Ok(())
    }

    /// 测试创建文件
    ///
    /// ## 测试目的
    /// 验证 `CliTestEnv::create_file()` 方法能够成功创建文件，并写入指定内容。
    ///
    /// ## 测试场景
    /// 1. 创建CliTestEnv
    /// 2. 创建文件（test.txt）
    /// 3. 验证文件存在
    /// 4. 验证文件内容正确
    ///
    /// ## 预期结果
    /// - 文件创建成功
    /// - 文件内容与预期一致
    #[test]
    fn test_create_file() -> Result<()> {
        let env = CliTestEnv::new()?;
        let path = env.path();
        env.create_file("test.txt", "test content")?;

        let file_path = path.join("test.txt");
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path)?;
        assert_eq!(content, "test content");

        Ok(())
    }

    /// 测试创建配置文件
    ///
    /// ## 测试目的
    /// 验证 `CliTestEnv::create_config()` 方法能够成功创建配置文件（.workflow/workflow.toml），并写入指定内容。
    ///
    /// ## 测试场景
    /// 1. 创建CliTestEnv
    /// 2. 创建配置文件
    /// 3. 验证配置文件存在
    /// 4. 验证配置文件内容包含预期内容
    ///
    /// ## 预期结果
    /// - 配置文件创建成功
    /// - 配置文件路径为.workflow/workflow.toml
    /// - 配置文件内容包含预期内容
    #[test]
    fn test_create_config() -> Result<()> {
        let env = CliTestEnv::new()?;
        env.create_config("[jira]\nurl = \"test\"")?;

        let config_path = env.path().join(".workflow").join("workflow.toml");
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path)?;
        assert!(content.contains("jira"));

        Ok(())
    }
}

