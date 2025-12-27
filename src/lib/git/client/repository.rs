//! Git 仓库封装
//!
//! 提供统一的 Git 仓库操作接口，封装 git2::Repository 的所有常用操作。

use color_eyre::{eyre::WrapErr, Result};
use git2::{FetchOptions, PushOptions, Repository, Signature};
use std::path::Path;

use super::remote::GitRemote;
use crate::git::GitAuth;

/// Git 仓库封装
///
/// 提供统一的 Git 仓库操作接口，封装 git2::Repository 的所有常用操作。
pub struct GitRepository {
    inner: Repository,
}

impl GitRepository {
    /// 打开当前目录的 Git 仓库
    ///
    /// 从当前工作目录开始向上查找 `.git` 目录，打开 Git 仓库。
    ///
    /// # 返回
    ///
    /// 返回打开的 `GitRepository` 对象。
    ///
    /// # 错误
    ///
    /// 如果不在 Git 仓库中或打开失败，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::git::GitRepository;
    /// # use color_eyre::Result;
    /// # fn main() -> Result<()> {
    /// let repo = GitRepository::open()?;
    /// let branch_name = repo.current_branch_name()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open() -> Result<Self> {
        let repo = Repository::open(".")
            .wrap_err("Failed to open Git repository. Make sure you're in a Git repository.")?;
        Ok(Self { inner: repo })
    }

    /// 打开指定路径的 Git 仓库
    ///
    /// 从指定路径开始向上查找 `.git` 目录，打开 Git 仓库。
    ///
    /// # 参数
    ///
    /// * `path` - 仓库路径（可以是仓库根目录或子目录）
    ///
    /// # 返回
    ///
    /// 返回打开的 `GitRepository` 对象。
    ///
    /// # 错误
    ///
    /// 如果不在 Git 仓库中或打开失败，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::git::GitRepository;
    /// # use color_eyre::Result;
    /// # fn main() -> Result<()> {
    /// let repo = GitRepository::open_at("/path/to/repo")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_at(path: impl AsRef<Path>) -> Result<Self> {
        let repo = Repository::open(path.as_ref())
            .wrap_err_with(|| format!("Failed to open Git repository at: {:?}", path.as_ref()))?;
        Ok(Self { inner: repo })
    }

    /// 初始化 Git 仓库
    ///
    /// 在指定路径初始化一个新的 Git 仓库。如果路径已存在 Git 仓库，会先删除现有的 `.git` 目录。
    ///
    /// # 参数
    ///
    /// * `path` - 仓库路径
    /// * `initial_branch` - 初始分支名（默认为 "main"）
    ///
    /// # 返回
    ///
    /// 返回初始化的 `GitRepository` 对象。
    ///
    /// # 错误
    ///
    /// 如果初始化失败，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::git::GitRepository;
    /// # use color_eyre::Result;
    /// # fn main() -> Result<()> {
    /// let repo = GitRepository::init("/path/to/repo", Some("main"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn init(path: impl AsRef<Path>, initial_branch: Option<&str>) -> Result<Self> {
        let path = path.as_ref();
        let initial_branch = initial_branch.unwrap_or("main");

        // 删除现有 .git 目录（如果存在）
        let git_dir = path.join(".git");
        if git_dir.exists() {
            std::fs::remove_dir_all(&git_dir).map_err(|e| {
                color_eyre::eyre::eyre!("Failed to remove existing .git directory: {}", e)
            })?;
        }

        // 初始化仓库
        let mut init_opts = git2::RepositoryInitOptions::new();
        init_opts.initial_head(initial_branch);
        let repo = Repository::init_opts(path, &init_opts)
            .wrap_err("Failed to initialize git repository")?;

        Ok(Self { inner: repo })
    }

    /// 初始化 Git 仓库并创建初始提交
    ///
    /// 在指定路径初始化一个新的 Git 仓库，配置用户信息，并创建初始提交。
    /// 如果路径已存在 Git 仓库，会先删除现有的 `.git` 目录。
    ///
    /// # 参数
    ///
    /// * `path` - 仓库路径
    /// * `initial_branch` - 初始分支名（默认为 "main"）
    /// * `user_name` - Git 用户名称（默认为 "Test User"）
    /// * `user_email` - Git 用户邮箱（默认为 "test@example.com"）
    /// * `initial_file` - 初始文件名（默认为 "README.md"）
    /// * `initial_content` - 初始文件内容（默认为 "# Test Repository\n"）
    /// * `commit_message` - 初始提交消息（默认为 "Initial commit"）
    ///
    /// # 返回
    ///
    /// 返回初始化的 `GitRepository` 对象。
    ///
    /// # 错误
    ///
    /// 如果初始化或提交失败，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::git::GitRepository;
    /// # use color_eyre::Result;
    /// # fn main() -> Result<()> {
    /// let repo = GitRepository::init_with_commit(
    ///     "/path/to/repo",
    ///     Some("main"),
    ///     Some("Test User"),
    ///     Some("test@example.com"),
    ///     None,
    ///     None,
    ///     None,
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn init_with_commit(
        path: impl AsRef<Path>,
        initial_branch: Option<&str>,
        user_name: Option<&str>,
        user_email: Option<&str>,
        initial_file: Option<&str>,
        initial_content: Option<&str>,
        commit_message: Option<&str>,
    ) -> Result<Self> {
        let path = path.as_ref();
        let initial_branch = initial_branch.unwrap_or("main");
        let user_name = user_name.unwrap_or("Test User");
        let user_email = user_email.unwrap_or("test@example.com");
        let initial_file = initial_file.unwrap_or("README.md");
        let initial_content = initial_content.unwrap_or("# Test Repository\n");
        let commit_message = commit_message.unwrap_or("Initial commit");

        // 初始化仓库
        let mut repo = Self::init(path, Some(initial_branch))?;

        // 配置用户（本地配置）
        let mut config = repo.as_inner().config().wrap_err("Failed to open repository config")?;
        config.set_str("user.name", user_name).wrap_err("Failed to set user.name")?;
        config.set_str("user.email", user_email).wrap_err("Failed to set user.email")?;

        // 创建初始文件
        std::fs::write(path.join(initial_file), initial_content)
            .wrap_err("Failed to write initial file")?;

        // 添加所有文件到索引
        let tree_id = {
            let mut index = repo.as_inner().index().wrap_err("Failed to open repository index")?;
            index
                .add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)
                .wrap_err("Failed to add files to index")?;
            let tree_id = index.write_tree().wrap_err("Failed to write index to tree")?;
            index.write().wrap_err("Failed to write index")?;
            tree_id
        };

        // 创建提交
        let signature =
            git2::Signature::now(user_name, user_email).wrap_err("Failed to create signature")?;
        {
            let repo_inner = repo.as_inner_mut();
            let tree = repo_inner.find_tree(tree_id).wrap_err("Failed to find tree")?;
            repo_inner
                .commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    commit_message,
                    &tree,
                    &[],
                )
                .wrap_err("Failed to create initial commit")?;
        }

        Ok(repo)
    }

    /// 获取仓库签名（作者信息）
    ///
    /// 从 Git 配置中读取用户签名信息（name 和 email）。
    ///
    /// # 返回
    ///
    /// 返回 `Signature` 对象，包含用户名和邮箱。
    ///
    /// # 错误
    ///
    /// 如果无法获取签名信息，返回相应的错误信息。
    pub fn signature(&self) -> Result<Signature<'_>> {
        self.inner.signature().wrap_err("Failed to get repository signature")
    }

    /// 查找 origin 远程仓库
    ///
    /// 查找并返回名为 "origin" 的远程仓库。
    ///
    /// # 返回
    ///
    /// 返回 `GitRemote` 对象。
    ///
    /// # 错误
    ///
    /// 如果找不到 "origin" 远程仓库，返回相应的错误信息。
    ///
    /// # 注意
    ///
    /// 返回的 `GitRemote` 的生命周期与 `GitRepository` 相关。
    pub fn find_origin_remote(&mut self) -> Result<GitRemote<'_>> {
        let remote = self.inner.find_remote("origin").wrap_err("Failed to find remote 'origin'")?;
        Ok(GitRemote::new(remote))
    }

    /// 查找指定名称的远程仓库
    ///
    /// # 参数
    ///
    /// * `name` - 远程仓库名称（如 "origin", "upstream"）
    ///
    /// # 返回
    ///
    /// 返回 `GitRemote` 对象。
    ///
    /// # 错误
    ///
    /// 如果找不到指定名称的远程仓库，返回相应的错误信息。
    ///
    /// # 注意
    ///
    /// 返回的 `GitRemote` 的生命周期与 `GitRepository` 相关。
    pub fn find_remote(&mut self, name: &str) -> Result<GitRemote<'_>> {
        let remote = self
            .inner
            .find_remote(name)
            .wrap_err_with(|| format!("Failed to find remote '{}'", name))?;
        Ok(GitRemote::new(remote))
    }

    /// 获取 HEAD 引用
    ///
    /// # 返回
    ///
    /// 返回 HEAD 引用对象。
    ///
    /// # 错误
    ///
    /// 如果无法获取 HEAD 引用，返回相应的错误信息。
    pub fn head(&self) -> Result<git2::Reference<'_>> {
        self.inner.head().wrap_err("Failed to get HEAD reference")
    }

    /// 获取当前分支名
    ///
    /// 从 HEAD 引用中提取当前分支名称。
    ///
    /// # 返回
    ///
    /// 返回当前分支名称（不包含 `refs/heads/` 前缀）。
    ///
    /// # 错误
    ///
    /// 如果 HEAD 不是指向分支（如 detached HEAD 状态），返回相应的错误信息。
    pub fn current_branch_name(&self) -> Result<String> {
        let head = self.head()?;
        head.name()
            .and_then(|name| name.strip_prefix("refs/heads/"))
            .ok_or_else(|| color_eyre::eyre::eyre!("HEAD is not pointing to a branch"))
            .map(|s| s.to_string())
    }

    /// 查找引用
    ///
    /// # 参数
    ///
    /// * `name` - 引用名称（如 "refs/heads/main", "refs/remotes/origin/main"）
    ///
    /// # 返回
    ///
    /// 返回找到的引用对象。
    ///
    /// # 错误
    ///
    /// 如果找不到指定引用，返回相应的错误信息。
    pub fn find_reference(&self, name: &str) -> Result<git2::Reference<'_>> {
        self.inner
            .find_reference(name)
            .wrap_err_with(|| format!("Failed to find reference '{}'", name))
    }

    /// 获取索引
    ///
    /// # 返回
    ///
    /// 返回仓库的索引对象。
    ///
    /// # 错误
    ///
    /// 如果无法获取索引，返回相应的错误信息。
    pub fn index(&mut self) -> Result<git2::Index> {
        self.inner.index().wrap_err("Failed to get repository index")
    }

    /// 获取配置的 FetchOptions（包含认证）
    ///
    /// 创建一个新的 `FetchOptions` 对象，并配置好认证回调。
    ///
    /// # 返回
    ///
    /// 返回配置好的 `FetchOptions` 对象。
    pub fn get_fetch_options() -> FetchOptions<'static> {
        let mut options = FetchOptions::new();
        options.remote_callbacks(GitAuth::get_remote_callbacks());
        options
    }

    /// 获取配置的 PushOptions（包含认证）
    ///
    /// 创建一个新的 `PushOptions` 对象，并配置好认证回调。
    ///
    /// # 返回
    ///
    /// 返回配置好的 `PushOptions` 对象。
    pub fn get_push_options() -> PushOptions<'static> {
        let mut options = PushOptions::new();
        options.remote_callbacks(GitAuth::get_remote_callbacks());
        options
    }

    /// 逃生舱：直接访问底层 Repository
    ///
    /// 用于需要直接使用 git2 高级功能的场景。
    ///
    /// # 返回
    ///
    /// 返回底层 `Repository` 的不可变引用。
    pub fn as_inner(&self) -> &Repository {
        &self.inner
    }

    /// 逃生舱：可变访问底层 Repository
    ///
    /// 用于需要直接使用 git2 高级功能的场景。
    ///
    /// # 返回
    ///
    /// 返回底层 `Repository` 的可变引用。
    pub fn as_inner_mut(&mut self) -> &mut Repository {
        &mut self.inner
    }
}
