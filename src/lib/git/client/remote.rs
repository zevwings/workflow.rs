//! Git 远程仓库封装
//!
//! 提供统一的 Git 远程仓库操作接口，封装 git2::Remote 的常用操作。

use color_eyre::{eyre::WrapErr, Result};
use git2::{FetchOptions, PushOptions, Remote};

/// Git 远程仓库封装
///
/// 提供统一的 Git 远程仓库操作接口，封装 git2::Remote 的常用操作。
///
/// 注意：`GitRemote` 持有 `Remote` 的所有权，但 `Remote` 本身可能持有对 `Repository` 的引用。
/// 因此，`GitRemote` 的生命周期与创建它的 `GitRepository` 相关。
pub struct GitRemote<'repo> {
    inner: Remote<'repo>,
}

impl<'repo> GitRemote<'repo> {
    /// 创建新的 GitRemote 实例
    ///
    /// 这是一个内部方法，通常通过 `GitRepository::find_remote()` 或
    /// `GitRepository::find_origin_remote()` 来创建。
    pub(crate) fn new(remote: Remote<'repo>) -> Self {
        Self { inner: remote }
    }

    /// 获取远程 URL
    ///
    /// # 返回
    ///
    /// 返回远程仓库的 URL，如果未设置则返回 `None`。
    pub fn url(&self) -> Option<&str> {
        self.inner.url()
    }

    /// 推送到远程
    ///
    /// # 参数
    ///
    /// * `refspecs` - 要推送的引用规范数组（如 `["refs/heads/main:refs/heads/main"]`）
    /// * `options` - 可选的推送选项（包含认证信息）
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
    /// let mut push_options = GitRepository::get_push_options();
    /// remote.push(&["refs/heads/main:refs/heads/main"], Some(&mut push_options))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn push(&mut self, refspecs: &[&str], options: Option<&mut PushOptions>) -> Result<()> {
        self.inner.push(refspecs, options).wrap_err("Failed to push to remote")
    }

    /// 从远程获取
    ///
    /// # 参数
    ///
    /// * `refspecs` - 要获取的引用规范数组（如 `["refs/heads/*:refs/remotes/origin/*"]`）
    /// * `options` - 可选的获取选项（包含认证信息）
    /// * `reflog_message` - 可选的 reflog 消息
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
    /// let mut fetch_options = GitRepository::get_fetch_options();
    /// remote.fetch(&["refs/heads/*:refs/remotes/origin/*"], Some(&mut fetch_options), None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn fetch(
        &mut self,
        refspecs: &[&str],
        options: Option<&mut FetchOptions>,
        reflog_message: Option<&str>,
    ) -> Result<()> {
        self.inner
            .fetch(refspecs, options, reflog_message)
            .wrap_err("Failed to fetch from remote")
    }

    /// 逃生舱：直接访问底层 Remote
    ///
    /// 用于需要直接使用 git2 高级功能的场景。
    ///
    /// # 返回
    ///
    /// 返回底层 `Remote` 的不可变引用。
    pub fn as_inner(&self) -> &Remote<'repo> {
        &self.inner
    }

    /// 逃生舱：可变访问底层 Remote
    ///
    /// 用于需要直接使用 git2 高级功能的场景。
    ///
    /// # 返回
    ///
    /// 返回底层 `Remote` 的可变引用。
    pub fn as_inner_mut(&mut self) -> &mut Remote<'repo> {
        &mut self.inner
    }
}
