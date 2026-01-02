//! Git 远程仓库封装
//!
//! 提供统一的 Git 远程仓库操作接口，使用 GitCommand 执行 git 命令。

use color_eyre::{eyre::WrapErr, Result};
use std::path::PathBuf;

use crate::git::commands::command::GitCommand;
use crate::git::commands::GitRepoCommand;

/// Git 远程仓库封装
///
/// 提供统一的 Git 远程仓库操作接口，使用 GitCommand 执行 git 命令。
pub struct GitRemote {
    name: String,
    repo_path: PathBuf,
}

impl GitRemote {
    /// 创建新的 GitRemote 实例
    ///
    /// 这是一个内部方法，通常通过 `GitRepository::find_remote()` 或
    /// `GitRepository::find_origin_remote()` 来创建。
    pub(crate) fn new(name: String, repo_path: PathBuf) -> Self {
        Self { name, repo_path }
    }

    /// 获取远程 URL
    ///
    /// # 返回
    ///
    /// 返回远程仓库的 URL，如果未设置则返回 `None`。
    pub fn url(&self) -> Result<String> {
        GitRepoCommand::get_remote_url(Some(&self.name), Some(&self.repo_path))
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get remote URL: {}", e))
    }

    /// 推送到远程
    ///
    /// # 参数
    ///
    /// * `refspecs` - 要推送的引用规范数组（如 `["refs/heads/main:refs/heads/main"]` 或 `["main"]`）
    ///
    /// # 返回
    ///
    /// 推送成功返回 `Ok(())`。
    ///
    /// # 错误
    ///
    /// 如果推送失败，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::git::{GitRepository, GitRemote};
    /// # use color_eyre::Result;
    /// # fn main() -> Result<()> {
    /// let mut repo = GitRepository::open()?;
    /// let mut remote = repo.find_origin_remote()?;
    /// remote.push(&["main"])?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn push(&mut self, refspecs: &[&str]) -> Result<()> {
        // 构建 git push 命令
        let mut args = vec!["push", &self.name];

        // 添加 refspecs
        for refspec in refspecs {
            args.push(refspec);
        }

        GitCommand::execute(&args, Some(&self.repo_path))
            .map_err(GitCommand::handle_auth_error)
            .wrap_err("Failed to push to remote")
    }

    /// 从远程获取
    ///
    /// # 参数
    ///
    /// * `refspecs` - 要获取的引用规范数组（如 `["refs/heads/*:refs/remotes/origin/*"]`）
    ///   如果为空，则获取所有默认引用
    ///
    /// # 返回
    ///
    /// 获取成功返回 `Ok(())`。
    ///
    /// # 错误
    ///
    /// 如果获取失败，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::git::{GitRepository, GitRemote};
    /// # use color_eyre::Result;
    /// # fn main() -> Result<()> {
    /// let mut repo = GitRepository::open()?;
    /// let mut remote = repo.find_origin_remote()?;
    /// remote.fetch(&[])?; // 获取所有默认引用
    /// # Ok(())
    /// # }
    /// ```
    pub fn fetch(&mut self, refspecs: &[&str]) -> Result<()> {
        // 如果没有指定 refspecs，使用简单的 fetch
        if refspecs.is_empty() {
            return GitRepoCommand::fetch(Some(&self.name), Some(&self.repo_path))
                .wrap_err("Failed to fetch from remote");
        }

        // 构建 git fetch 命令（带 refspecs）
        let mut args = vec!["fetch", &self.name];

        // 添加 refspecs
        for refspec in refspecs {
            args.push(refspec);
        }

        GitCommand::execute(&args, Some(&self.repo_path))
            .map_err(GitCommand::handle_auth_error)
            .wrap_err("Failed to fetch from remote")
    }

    /// 列出远程引用
    ///
    /// 使用 `git ls-remote` 命令列出远程仓库的所有引用。
    ///
    /// # 返回
    ///
    /// 返回元组向量 `(ref_name, sha)`，包含引用名称和对应的 SHA。
    ///
    /// # 错误
    ///
    /// 如果获取失败，返回相应的错误信息。
    pub fn list(&self) -> Result<Vec<(String, String)>> {
        let output = GitRepoCommand::ls_remote(&self.name, Some(&self.repo_path))
            .map_err(|e| color_eyre::eyre::eyre!("Failed to list remote references: {}", e))?;

        let refs: Vec<(String, String)> = GitCommand::parse_key_value(&output, '\t')
            .into_iter()
            .map(|(sha, ref_name)| (ref_name, sha))
            .collect();

        Ok(refs)
    }
}
