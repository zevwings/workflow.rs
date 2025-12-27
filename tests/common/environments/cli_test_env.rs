#![allow(clippy::test_attr_in_doctest)]

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
//! fn test_cli_command_return_ok() -> color_eyre::Result<()> {
//!     let env = CliTestEnv::new()?;
//!     env.init_git_repo()?;
//!     env.create_file("test.txt", "content")?;
//!
//!     Ok(())
//! }
//! ```

use color_eyre::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::common::isolation::TestIsolation;

/// 统一的CLI测试环境
///
/// 基于`TestIsolation`构建，提供完全隔离的CLI测试环境，包括：
/// - 独立的工作目录
/// - 隔离的环境变量
/// - 独立的Git配置（可选）
/// - 便捷的文件和配置管理
/// - 统一的项目路径和用户路径管理
///
/// # 功能特性
///
/// - ✅ 完全隔离的测试环境
/// - ✅ 支持Git仓库初始化
/// - ✅ 便捷的文件和配置管理
/// - ✅ RAII模式自动清理
/// - ✅ 统一的项目路径和用户路径（HOME）管理
/// - ✅ 自动禁用 iCloud，使用临时目录
pub struct CliTestEnv {
    /// 测试隔离管理器
    isolation: TestIsolation,
    /// 项目路径（仓库根目录，用于项目级配置）
    project_path: PathBuf,
    /// 用户路径（HOME 目录，用于用户级配置）
    home_path: PathBuf,
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
        let mut isolation = TestIsolation::new()?;

        // 创建两个独立的路径
        // - project_path: 用于项目级配置（仓库根目录）
        // - home_path: 用于用户级配置（HOME 目录）
        let project_path = isolation.work_dir().join("repo");
        let home_path = isolation.work_dir().join("home");
        fs::create_dir_all(&project_path)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create project path: {}", e))?;
        fs::create_dir_all(&home_path)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create home path: {}", e))?;

        // 统一设置环境变量
        // - HOME: 指向临时目录下的 home 目录
        // - WORKFLOW_DISABLE_ICLOUD: 禁用 iCloud，确保使用临时目录
        isolation.env_guard().set("HOME", &home_path.to_string_lossy());
        isolation.env_guard().set("WORKFLOW_DISABLE_ICLOUD", "1");

        Ok(Self {
            isolation,
            project_path,
            home_path,
        })
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
    /// 获取项目路径（用于项目级配置）
    ///
    /// 返回项目临时路径，用于项目级配置文件（如 `.workflow/config.toml`）。
    ///
    /// # 返回
    ///
    /// 返回项目路径的引用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::CliTestEnv;
    ///
    /// let env = CliTestEnv::new()?;
    /// let project_path = env.project_path();
    /// ```
    pub fn project_path(&self) -> &Path {
        &self.project_path
    }

    /// 获取用户路径（用于用户级配置）
    ///
    /// 返回用户临时路径（HOME），用于用户级配置文件（如 `~/.workflow/config/repository.toml`）。
    ///
    /// # 返回
    ///
    /// 返回用户路径的引用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::CliTestEnv;
    ///
    /// let env = CliTestEnv::new()?;
    /// let home_path = env.home_path();
    /// ```
    pub fn home_path(&self) -> &Path {
        &self.home_path
    }

    pub fn init_git_repo(&self) -> Result<&Self> {
        let work_dir = &self.project_path;

        // 确保.git目录不存在（如果存在则删除）
        let git_dir = work_dir.join(".git");
        if git_dir.exists() {
            std::fs::remove_dir_all(&git_dir).map_err(|e| {
                color_eyre::eyre::eyre!("Failed to remove existing .git directory: {}", e)
            })?;
        }

        let output = std::process::Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(work_dir)
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to init git repo: {}", e))?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!(
                "Failed to init git repo: {}",
                error
            ));
        }

        // 在仓库的配置文件中设置Git用户配置
        // 在 Command 中显式移除 GIT_CONFIG 环境变量，然后使用 --local 选项设置配置
        // 这样可以避免 "only one config file at a time" 错误
        // 设置用户配置，在 Command 中显式移除 GIT_CONFIG 环境变量
        let output = std::process::Command::new("git")
            .args(["config", "--local", "user.name", "Test User"])
            .current_dir(work_dir)
            .env_remove("GIT_CONFIG")
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to set git user name: {}", e))?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!(
                "Failed to set git user name: {}",
                error
            ));
        }

        let output = std::process::Command::new("git")
            .args(["config", "--local", "user.email", "test@example.com"])
            .current_dir(work_dir)
            .env_remove("GIT_CONFIG")
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to set git user email: {}", e))?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!(
                "Failed to set git user email: {}",
                error
            ));
        }

        // 禁用网络连接，避免测试超时
        // 设置 GIT_TERMINAL_PROMPT=0 和 url.insteadOf 来避免网络请求
        std::process::Command::new("git")
            .args([
                "config",
                "url.insteadOf",
                "https://github.com/test/test-repo.git",
            ])
            .current_dir(work_dir)
            .output()
            .ok(); // 允许失败

        // 添加remote origin（用于测试需要remote的功能）
        std::process::Command::new("git")
            .args([
                "remote",
                "add",
                "origin",
                "https://github.com/test/test-repo.git",
            ])
            .current_dir(work_dir)
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to add remote origin: {}", e))?;

        // 创建假的远程分支引用（让get_default_branch()等函数能正常工作）
        self.setup_fake_remote_refs()?;

        Ok(self)
    }

    /// 设置假的远程引用（用于测试需要远程分支的功能）
    ///
    /// 创建假的远程引用，让 `get_default_branch()` 等函数能正常工作，
    /// 但不进行真实的网络连接。
    ///
    /// # 返回
    ///
    /// 成功时返回`&Self`以支持链式调用，失败时返回错误
    ///
    /// # 功能
    ///
    /// 1. 创建假的远程分支引用（`refs/remotes/origin/main`）
    /// 2. 删除可能存在的旧引用（如`origin/master`）
    /// 3. 设置远程HEAD引用（`refs/remotes/origin/HEAD`）
    pub fn setup_fake_remote_refs(&self) -> Result<&Self> {
        let work_dir = &self.project_path;

        // 1. 创建假的远程分支引用（指向当前HEAD）
        std::process::Command::new("git")
            .args(["update-ref", "refs/remotes/origin/main", "HEAD"])
            .current_dir(work_dir)
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create remote ref: {}", e))?;

        // 2. 删除可能存在的旧引用（如origin/master）
        std::process::Command::new("git")
            .args(["update-ref", "-d", "refs/remotes/origin/master"])
            .current_dir(work_dir)
            .output()
            .ok(); // 允许失败，因为可能不存在

        // 3. 设置远程HEAD引用指向main（让 git remote show origin 能工作）
        std::process::Command::new("git")
            .args([
                "symbolic-ref",
                "refs/remotes/origin/HEAD",
                "refs/remotes/origin/main",
            ])
            .current_dir(work_dir)
            .output()
            .ok(); // 允许失败，某些Git版本可能不支持

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
        let full_path = self.project_path.join(path);

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
        let output = std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&self.project_path)
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to add files: {}", e))?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!("Failed to add files: {}", error));
        }

        let output = std::process::Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.project_path)
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to commit: {}", e))?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!("Failed to commit: {}", error));
        }

        Ok(self)
    }

    /// 创建Git分支
    ///
    /// # 参数
    ///
    /// * `branch_name` - 分支名称
    ///
    /// # 返回
    ///
    /// 返回`&Self`以支持链式调用
    ///
    /// # 错误
    ///
    /// - 无法创建分支
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::CliTestEnv;
    ///
    /// let env = CliTestEnv::new()?;
    /// env.init_git_repo()?;
    /// env.create_branch("feature/test")?;
    /// ```
    pub fn create_branch(&self, branch_name: &str) -> Result<&Self> {
        let output = std::process::Command::new("git")
            .args(["branch", branch_name])
            .current_dir(&self.project_path)
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create branch: {}", e))?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!(
                "Failed to create branch '{}': {}",
                branch_name,
                error
            ));
        }

        Ok(self)
    }

    /// 切换Git分支
    ///
    /// # 参数
    ///
    /// * `branch_name` - 分支名称
    ///
    /// # 返回
    ///
    /// 返回`&Self`以支持链式调用
    ///
    /// # 错误
    ///
    /// - 无法切换分支
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::CliTestEnv;
    ///
    /// let env = CliTestEnv::new()?;
    /// env.init_git_repo()?;
    /// env.create_branch("feature/test")?;
    /// env.checkout("feature/test")?;
    /// ```
    pub fn checkout(&self, branch_name: &str) -> Result<&Self> {
        let output = std::process::Command::new("git")
            .args(["checkout", branch_name])
            .current_dir(&self.project_path)
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to checkout branch: {}", e))?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!(
                "Failed to checkout branch '{}': {}",
                branch_name,
                error
            ));
        }

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
        let config_dir = self.project_path.join(".workflow");

        fs::create_dir_all(&config_dir)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create config directory: {}", e))?;

        let config_file = config_dir.join("workflow.toml");
        fs::write(&config_file, content)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to write config file: {}", e))?;

        Ok(self)
    }

    /// 创建项目级配置文件
    ///
    /// 在项目路径下创建 `.workflow/config.toml` 配置文件（项目级配置）。
    ///
    /// # 参数
    ///
    /// * `content` - 配置文件内容
    ///
    /// # 返回
    ///
    /// 返回创建的配置文件路径
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::CliTestEnv;
    ///
    /// let env = CliTestEnv::new()?;
    /// let config_path = env.create_project_config(r#"[template.commit]
    /// type = "conventional"
    /// "#)?;
    /// ```
    pub fn create_project_config(&self, content: &str) -> Result<PathBuf> {
        let config_dir = self.project_path.join(".workflow");
        fs::create_dir_all(&config_dir).map_err(|e| {
            color_eyre::eyre::eyre!("Failed to create project config directory: {}", e)
        })?;

        let config_file = config_dir.join("config.toml");
        fs::write(&config_file, content)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to write project config file: {}", e))?;

        Ok(config_file)
    }

    /// 创建用户级配置文件
    ///
    /// 在用户路径（HOME）下创建 `.workflow/config/repository.toml` 配置文件（用户级配置）。
    ///
    /// # 参数
    ///
    /// * `content` - 配置文件内容
    ///
    /// # 返回
    ///
    /// 返回创建的配置文件路径
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::CliTestEnv;
    ///
    /// let env = CliTestEnv::new()?;
    /// let config_path = env.create_home_config(r#"[repo_id]
    /// configured = true
    /// "#)?;
    /// ```
    pub fn create_home_config(&self, content: &str) -> Result<PathBuf> {
        let config_dir = self.home_path.join(".workflow").join("config");
        fs::create_dir_all(&config_dir).map_err(|e| {
            color_eyre::eyre::eyre!("Failed to create home config directory: {}", e)
        })?;

        let config_file = config_dir.join("repository.toml");
        fs::write(&config_file, content)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to write home config file: {}", e))?;

        Ok(config_file)
    }

    /// 创建用户级 workflow.toml 配置文件
    ///
    /// 在用户路径（HOME）下创建 `.workflow/config/workflow.toml` 配置文件（主配置文件）。
    /// 用于设置 LLM、Jira、GitHub 等全局配置。
    ///
    /// # 参数
    ///
    /// * `content` - 配置文件内容
    ///
    /// # 返回
    ///
    /// 返回创建的配置文件路径
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::CliTestEnv;
    ///
    /// let env = CliTestEnv::new()?;
    /// let config_path = env.create_home_workflow_config(r#"[llm]
    /// provider = "invalid_provider"
    /// "#)?;
    /// ```
    pub fn create_home_workflow_config(&self, content: &str) -> Result<PathBuf> {
        let config_dir = self.home_path.join(".workflow").join("config");
        fs::create_dir_all(&config_dir).map_err(|e| {
            color_eyre::eyre::eyre!("Failed to create home config directory: {}", e)
        })?;

        let config_file = config_dir.join("workflow.toml");
        fs::write(&config_file, content).map_err(|e| {
            color_eyre::eyre::eyre!("Failed to write home workflow config file: {}", e)
        })?;

        Ok(config_file)
    }

    /// 获取临时目录路径（向后兼容）
    ///
    /// 返回项目路径（仓库根目录），用于向后兼容。
    /// 新代码应该使用 `project_path()` 方法。
    ///
    /// # 返回
    ///
    /// 返回项目路径
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tests::common::environments::CliTestEnv;
    ///
    /// let env = CliTestEnv::new()?;
    /// let path = env.path();  // 返回 project_path
    /// ```
    pub fn path(&self) -> PathBuf {
        self.project_path.to_path_buf()
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
    fn test_cli_test_env_creation_return_ok() -> Result<()> {
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
    fn test_init_git_repo_return_ok() -> Result<()> {
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

    /// 测试创建分支
    ///
    /// ## 测试目的
    /// 验证 `CliTestEnv::create_branch()` 方法能够成功创建Git分支。
    ///
    /// ## 测试场景
    /// 1. 创建CliTestEnv
    /// 2. 初始化Git仓库
    /// 3. 创建初始提交（Git需要至少一个提交才能创建分支）
    /// 4. 创建分支
    /// 5. 验证分支存在
    ///
    /// ## 预期结果
    /// - 分支创建成功
    /// - 分支存在于Git仓库中
    #[test]
    #[serial]
    fn test_create_branch_return_ok() -> Result<()> {
        let env = CliTestEnv::new()?;
        env.init_git_repo()?;

        // 创建初始提交（Git需要至少一个提交才能创建分支）
        env.create_file("README.md", "# Test")?;
        env.create_commit("Initial commit")?;

        env.create_branch("feature/test")?;

        // 验证分支存在
        let output = std::process::Command::new("git")
            .args(["branch", "--list", "feature/test"])
            .current_dir(env.path())
            .output()?;

        let branch_list = String::from_utf8_lossy(&output.stdout);
        assert!(branch_list.contains("feature/test"));

        Ok(())
    }

    /// 测试切换分支
    ///
    /// ## 测试目的
    /// 验证 `CliTestEnv::checkout()` 方法能够成功切换Git分支。
    ///
    /// ## 测试场景
    /// 1. 创建CliTestEnv
    /// 2. 初始化Git仓库
    /// 3. 创建分支
    /// 4. 切换分支
    /// 5. 验证当前分支
    ///
    /// ## 预期结果
    /// - 分支切换成功
    /// - 当前分支为切换后的分支
    #[test]
    #[serial]
    fn test_checkout_branch_return_ok() -> Result<()> {
        let env = CliTestEnv::new()?;
        env.init_git_repo()?;

        // 创建初始提交（checkout需要至少一个提交）
        env.create_file("README.md", "# Test")?;
        env.create_commit("Initial commit")?;

        env.create_branch("feature/test")?;
        env.checkout("feature/test")?;

        // 验证当前分支
        let output = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(env.path())
            .output()?;

        let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert_eq!(current_branch, "feature/test");

        Ok(())
    }
}
