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

use color_eyre::{eyre::WrapErr, Result};
use git2::{IndexAddOption, Repository, Signature};
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

        // 初始化Git仓库，设置默认分支为main
        let mut init_opts = git2::RepositoryInitOptions::new();
        init_opts.initial_head("main");
        let repo = Repository::init_opts(work_dir, &init_opts)
            .wrap_err("Failed to initialize git repository")?;

        // 在仓库的配置文件中设置Git用户配置
        // 使用 git2 API 设置本地配置，避免 GIT_CONFIG 环境变量冲突
        let mut config = repo.config().wrap_err("Failed to open repository config")?;
        config.set_str("user.name", "Test User").wrap_err("Failed to set user.name")?;
        config
            .set_str("user.email", "test@example.com")
            .wrap_err("Failed to set user.email")?;

        // 注意：不设置 url.insteadOf，因为这会替换 URL 为 file:///dev/null
        // 导致 extract_repo_name() 无法提取仓库名
        // 我们使用假的远程引用而不是替换 URL，这样既能避免网络请求，又能保持 URL 格式正确

        // 添加remote origin（用于测试需要remote的功能）
        repo.remote("origin", "https://github.com/test/test-repo.git")
            .wrap_err("Failed to add remote origin")?;

        // 创建初始提交（这样 HEAD 才会存在，setup_fake_remote_refs 才能正常工作）
        std::fs::write(work_dir.join("README.md"), "# Test Repository\n")?;

        // 添加所有文件到索引
        let mut index = repo.index().wrap_err("Failed to open repository index")?;
        index
            .add_all(["."].iter(), IndexAddOption::DEFAULT, None)
            .wrap_err("Failed to add files to index")?;
        let tree_id = index.write_tree().wrap_err("Failed to write index to tree")?;
        index.write().wrap_err("Failed to write index")?;

        // 创建提交
        let tree = repo.find_tree(tree_id).wrap_err("Failed to find tree")?;
        let signature = Signature::now("Test User", "test@example.com")
            .wrap_err("Failed to create signature")?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )
        .wrap_err("Failed to create initial commit")?;

        // 创建假的远程分支引用（让get_default_branch()等函数能正常工作）
        self.setup_fake_remote_refs()?;

        Ok(self)
    }

    /// 初始化Git仓库（不创建初始提交）
    ///
    /// 创建一个空的Git仓库，不包含任何提交。
    /// 用于测试需要空仓库的场景（如测试空仓库的错误处理）。
    ///
    /// # 返回
    ///
    /// 成功时返回`&Self`以支持链式调用，失败时返回错误
    pub fn init_git_repo_empty(&self) -> Result<&Self> {
        let work_dir = &self.project_path;

        // 确保.git目录不存在（如果存在则删除）
        let git_dir = work_dir.join(".git");
        if git_dir.exists() {
            std::fs::remove_dir_all(&git_dir).map_err(|e| {
                color_eyre::eyre::eyre!("Failed to remove existing .git directory: {}", e)
            })?;
        }

        // 初始化Git仓库，设置默认分支为main
        let mut init_opts = git2::RepositoryInitOptions::new();
        init_opts.initial_head("main");
        let repo = Repository::init_opts(work_dir, &init_opts)
            .wrap_err("Failed to initialize git repository")?;

        // 在仓库的配置文件中设置Git用户配置
        let mut config = repo.config().wrap_err("Failed to open repository config")?;
        config.set_str("user.name", "Test User").wrap_err("Failed to set user.name")?;
        config
            .set_str("user.email", "test@example.com")
            .wrap_err("Failed to set user.email")?;

        // 添加remote origin（用于测试需要remote的功能）
        repo.remote("origin", "https://github.com/test/test-repo.git")
            .wrap_err("Failed to add remote origin")?;

        // 注意：不创建初始提交，保持仓库为空
        // 也不调用 setup_fake_remote_refs()，因为 HEAD 不存在

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
        let repo = Repository::open(&self.project_path).wrap_err("Failed to open repository")?;

        // 1. 创建假的远程分支引用（指向当前HEAD）
        // 注意：如果仓库还没有提交（UnbornBranch），HEAD 可能不存在
        // 在这种情况下，我们跳过创建远程引用
        let head_oid = match repo.head() {
            Ok(head) => head
                .target()
                .ok_or_else(|| color_eyre::eyre::eyre!("HEAD does not point to a valid commit"))?,
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => {
                // 如果 HEAD 不存在（空仓库），跳过创建远程引用
                // 这通常发生在 init_git_repo() 被调用但还没有创建提交时
                return Ok(self);
            }
            Err(e) => {
                return Err(color_eyre::eyre::eyre!("Failed to get HEAD: {}", e.message()));
            }
        };

        repo.reference(
            "refs/remotes/origin/main",
            head_oid,
            true,
            "fake remote ref",
        )
        .wrap_err("Failed to create remote ref")?;

        // 2. 删除可能存在的旧引用（如origin/master）
        let old_ref = "refs/remotes/origin/master";
        if let Ok(mut reference) = repo.find_reference(old_ref) {
            let _ = reference.delete();
        }

        // 3. 设置远程HEAD引用指向main（让 git remote show origin 能工作）
        let _ = repo.reference_symbolic(
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
            true,
            "fake remote HEAD",
        );

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
        let repo = Repository::open(&self.project_path).wrap_err("Failed to open repository")?;

        // 添加所有文件到索引
        let mut index = repo.index().wrap_err("Failed to open repository index")?;
        index
            .add_all(["."].iter(), IndexAddOption::DEFAULT, None)
            .wrap_err("Failed to add files to index")?;
        let tree_id = index.write_tree().wrap_err("Failed to write index to tree")?;
        index.write().wrap_err("Failed to write index")?;

        // 创建提交
        let tree = repo.find_tree(tree_id).wrap_err("Failed to find tree")?;
        let signature = Signature::now("Test User", "test@example.com")
            .wrap_err("Failed to create signature")?;

        // 获取父提交（如果有）
        let parent_commit = repo
            .head()
            .ok()
            .and_then(|head| head.target().and_then(|oid| repo.find_commit(oid).ok()));
        let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )
        .wrap_err_with(|| format!("Failed to create commit: {}", message))?;

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
        let repo = Repository::open(&self.project_path).wrap_err("Failed to open repository")?;
        let head = repo.head().wrap_err("Failed to get HEAD")?;
        let head_commit = repo
            .find_commit(head.target().unwrap())
            .wrap_err("Failed to find HEAD commit")?;
        repo.branch(branch_name, &head_commit, false)
            .wrap_err_with(|| format!("Failed to create branch: {}", branch_name))?;
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
        let repo = Repository::open(&self.project_path).wrap_err("Failed to open repository")?;
        let refname = format!("refs/heads/{}", branch_name);
        repo.set_head(&refname)
            .wrap_err_with(|| format!("Failed to checkout branch: {}", branch_name))?;
        repo.checkout_head(Some(
            git2::build::CheckoutBuilder::default()
                .force()
                .remove_ignored(false)
                .remove_untracked(false),
        ))
        .wrap_err_with(|| format!("Failed to checkout HEAD for branch: {}", branch_name))?;
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
        let repo = Repository::open(env.path()).unwrap();
        let branch = repo.find_branch("feature/test", git2::BranchType::Local);
        assert!(branch.is_ok(), "Branch should exist");

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
        let repo = Repository::open(env.path()).unwrap();
        let head = repo.head().unwrap();
        let branch_name = head.name().unwrap();
        let branch_name = branch_name.strip_prefix("refs/heads/").unwrap();
        assert_eq!(branch_name, "feature/test");

        Ok(())
    }
}
