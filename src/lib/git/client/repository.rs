//! Git 仓库封装
//!
//! 提供统一的 Git 仓库操作接口，使用 GitCommand 执行 git 命令。

use color_eyre::{eyre::WrapErr, Result};
use std::path::{Path, PathBuf};

use super::remote::GitRemote;
use crate::git::commands::{GitBranchCommand, GitCommitCommand, GitConfigCommand, GitRepoCommand};

/// Git 仓库封装
///
/// 提供统一的 Git 仓库操作接口，使用 GitCommand 执行 git 命令。
pub struct GitRepository {
    path: PathBuf,
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
        // 使用 git rev-parse --show-toplevel 获取仓库根目录
        let workdir = GitRepoCommand::get_workdir(None).map_err(|e| match e {
            _ if e.to_string().contains("Not in a Git repository") => {
                color_eyre::eyre::eyre!(
                    "Not in a Git repository. Make sure you're in a Git repository."
                )
            }
            _ => color_eyre::eyre::eyre!("Failed to open Git repository: {}", e),
        })?;

        let path = PathBuf::from(workdir.trim());
        Ok(Self { path })
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
        let path_ref = path.as_ref();
        // 使用 git rev-parse --show-toplevel 获取仓库根目录
        let workdir = GitRepoCommand::get_workdir(Some(path_ref)).map_err(|e| {
            if e.to_string().contains("Not in a Git repository") {
                color_eyre::eyre::eyre!("Not in a Git repository at: {:?}", path_ref)
            } else {
                color_eyre::eyre::eyre!("Failed to open Git repository at: {:?}: {}", path_ref, e)
            }
        })?;

        let path = PathBuf::from(workdir.trim());
        Ok(Self { path })
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

        // 确保目录存在（在 Windows 上，如果目录不存在，git init 可能会失败）
        std::fs::create_dir_all(path).map_err(|e| {
            color_eyre::eyre::eyre!("Failed to create directory: {}", e)
        })?;

        // 删除现有 .git 目录（如果存在）
        let git_dir = path.join(".git");
        if git_dir.exists() {
            std::fs::remove_dir_all(&git_dir).map_err(|e| {
                color_eyre::eyre::eyre!("Failed to remove existing .git directory: {}", e)
            })?;
        }

        // 初始化仓库
        GitRepoCommand::init(Some(initial_branch), Some(path))
            .map_err(|e| color_eyre::eyre::eyre!("Failed to initialize git repository: {}", e))?;

        Ok(Self {
            path: path.to_path_buf(),
        })
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
        let repo = Self::init(path, Some(initial_branch))?;

        // 配置用户（本地配置）
        GitConfigCommand::set_local("user.name", user_name, Some(&repo.path))
            .wrap_err("Failed to set user.name")?;
        GitConfigCommand::set_local("user.email", user_email, Some(&repo.path))
            .wrap_err("Failed to set user.email")?;

        // 创建初始文件
        std::fs::write(path.join(initial_file), initial_content)
            .wrap_err("Failed to write initial file")?;

        // 添加所有文件并提交
        GitCommitCommand::add_all(Some(&repo.path)).wrap_err("Failed to add files to index")?;
        GitCommitCommand::commit(commit_message, false, Some(&repo.path))
            .wrap_err("Failed to create initial commit")?;

        Ok(repo)
    }

    /// 获取仓库签名（作者信息）
    ///
    /// 从 Git 配置中读取用户签名信息（name 和 email）。
    ///
    /// # 返回
    ///
    /// 返回元组 `(name, email)`，包含用户名和邮箱。
    ///
    /// # 错误
    ///
    /// 如果无法获取签名信息，返回相应的错误信息。
    pub fn signature(&self) -> Result<(String, String)> {
        let name = GitConfigCommand::get_local("user.name", Some(&self.path))
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get user.name: {}", e))?;

        let email = GitConfigCommand::get_local("user.email", Some(&self.path))
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get user.email: {}", e))?;

        Ok((name, email))
    }

    /// 查找 origin 远程仓库
    ///
    /// 查找并返回名为 "origin" 的远程仓库。
    /// 如果 URL 是简写的 SSH 格式，会自动规范化。
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
    pub fn find_origin_remote(&mut self) -> Result<GitRemote> {
        self.find_remote("origin")
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
    pub fn find_remote(&mut self, name: &str) -> Result<GitRemote> {
        // 检查远程是否存在
        let remotes = GitRepoCommand::list_remotes(Some(&self.path))
            .map_err(|e| color_eyre::eyre::eyre!("Failed to list remotes: {}", e))?;

        if !remotes.contains(&name.to_string()) {
            return Err(color_eyre::eyre::eyre!("Failed to find remote '{}'", name));
        }

        // 获取远程 URL
        let url = GitRepoCommand::get_remote_url(Some(name), Some(&self.path)).map_err(|e| {
            color_eyre::eyre::eyre!("Failed to get remote URL for '{}': {}", name, e)
        })?;

        // 规范化 SSH URL 格式（如果需要）
        // 注意：Git 命令本身支持简写 SSH URL，所以不需要规范化
        // 但为了保持兼容性，我们仍然检查是否需要规范化
        let normalized_url = Self::normalize_ssh_url(&url);
        if let Some(normalized) = normalized_url {
            GitRepoCommand::set_remote_url(name, &normalized, Some(&self.path))
                .wrap_err("Failed to normalize remote URL")?;
        }

        Ok(GitRemote::new(name.to_string(), self.path.clone()))
    }

    /// 规范化 SSH URL 格式
    ///
    /// 将简写的 SSH URL 格式 (`git@host:path`) 转换为完整格式 (`ssh://git@host/path`)。
    /// git2 库在某些情况下不支持简写格式，需要完整格式。
    ///
    /// # 参数
    ///
    /// * `url` - 远程仓库 URL
    ///
    /// # 返回
    ///
    /// 如果需要转换，返回转换后的 URL；如果不需要转换，返回 `None`。
    fn normalize_ssh_url(url: &str) -> Option<String> {
        // 检查是否是简写的 SSH URL 格式: git@host:path
        if url.starts_with("git@") && !url.starts_with("ssh://") {
            // 查找第一个冒号（分隔 host 和 path）
            if let Some(colon_pos) = url.find(':') {
                let host_part = &url[4..colon_pos]; // 跳过 "git@"
                let path_part = &url[colon_pos + 1..];
                // 转换为完整格式: ssh://git@host/path
                return Some(format!("ssh://git@{}/{}", host_part, path_part));
            }
        }
        None
    }

    /// 获取 HEAD 引用的 SHA
    ///
    /// # 返回
    ///
    /// 返回 HEAD 指向的提交 SHA。
    ///
    /// # 错误
    ///
    /// 如果无法获取 HEAD 引用，返回相应的错误信息。
    pub fn head(&self) -> Result<String> {
        GitCommitCommand::get_head_sha(Some(&self.path))
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get HEAD reference: {}", e))
    }

    /// 获取当前分支名
    ///
    /// 从 HEAD 引用中提取当前分支名称。
    ///
    /// # 返回
    ///
    /// 返回当前分支的名称（不包含 `refs/heads/` 前缀）。
    ///
    /// # 错误
    ///
    /// 如果 HEAD 不是指向分支（如 detached HEAD 状态），返回相应的错误信息。
    pub fn current_branch_name(&self) -> Result<String> {
        GitBranchCommand::current_branch(Some(&self.path))
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get current branch: {}", e))
    }

    /// 查找引用的 SHA
    ///
    /// # 参数
    ///
    /// * `name` - 引用名称（如 "refs/heads/main", "refs/remotes/origin/main", "HEAD"）
    ///
    /// # 返回
    ///
    /// 返回引用指向的提交 SHA。
    ///
    /// # 错误
    ///
    /// 如果找不到指定引用，返回相应的错误信息。
    pub fn find_reference(&self, name: &str) -> Result<String> {
        GitCommitCommand::rev_parse(name, Some(&self.path))
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find reference '{}': {}", name, e))
    }

    /// 获取仓库路径
    ///
    /// # 返回
    ///
    /// 返回仓库的根目录路径。
    pub fn path(&self) -> &Path {
        &self.path
    }
}
