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
        // 直接执行推送操作（移除超时和重试机制以避免线程创建问题）
        self.inner.push(refspecs, options).wrap_err("Failed to push to remote")?;

        Ok(())
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
        // 直接执行获取操作（移除超时和重试机制以避免线程创建问题）
        self.inner
            .fetch(refspecs, options, reflog_message)
            .wrap_err("Failed to fetch from remote")?;

        Ok(())
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

#[cfg(test)]
mod tests {
    use crate::git::GitRepository;
    use color_eyre::Result;
    use tempfile::TempDir;

    /// 测试查找 origin 远程仓库
    ///
    /// ## 测试目的
    /// 验证 GitRemote::find_origin_remote() 能够正确查找并返回 origin 远程仓库。
    ///
    /// ## 测试场景
    /// 1. 创建临时目录并初始化 Git 仓库
    /// 2. 添加 origin 远程仓库
    /// 3. 打开仓库并查找 origin 远程
    /// 4. 验证能够获取远程 URL
    ///
    /// ## 预期结果
    /// - 成功找到 origin 远程仓库
    /// - 远程 URL 与设置的 URL 一致
    #[test]
    fn test_find_origin_remote() -> Result<()> {
        // 创建一个临时目录并初始化 Git 仓库
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // 初始化 Git 仓库（使用 git2）
        let mut repo = GitRepository::init(repo_path, None)?;

        // 添加 origin 远程（使用 git2 API）
        repo.as_inner_mut()
            .remote("origin", "https://github.com/test/repo.git")
            .map_err(|e| color_eyre::eyre::eyre!("Failed to add remote: {}", e))?;

        // 打开仓库并查找 origin 远程
        let mut repo = GitRepository::open_at(repo_path)?;
        let remote = repo.find_origin_remote()?;

        // 验证能够获取 URL
        let url = remote.url();
        assert_eq!(url, Some("https://github.com/test/repo.git"));

        Ok(())
    }

    /// 测试查找不存在的远程仓库
    ///
    /// ## 测试目的
    /// 验证 GitRemote::find_remote() 在查找不存在的远程仓库时能够正确返回错误。
    ///
    /// ## 测试场景
    /// 1. 创建临时目录并初始化 Git 仓库（不添加任何远程）
    /// 2. 尝试查找不存在的远程仓库 "nonexistent"
    /// 3. 验证返回错误
    ///
    /// ## 预期结果
    /// - 查找不存在的远程仓库返回错误
    #[test]
    fn test_find_remote_not_found() -> Result<()> {
        // 创建一个临时目录并初始化 Git 仓库
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // 初始化 Git 仓库（使用 git2）
        let _repo = GitRepository::init(repo_path, None)?;

        // 打开仓库并尝试查找不存在的远程
        let mut repo = GitRepository::open_at(repo_path)?;
        let result = repo.find_remote("nonexistent");

        assert!(result.is_err());

        Ok(())
    }

    /// 测试获取远程 URL
    ///
    /// ## 测试目的
    /// 验证 GitRemote::url() 能够正确获取远程仓库的 URL，并处理 SSH URL 格式的规范化。
    ///
    /// ## 测试场景
    /// 1. 创建临时目录并初始化 Git 仓库
    /// 2. 添加使用 SSH 格式的 origin 远程（git@host:path）
    /// 3. 查找 origin 远程并获取 URL
    /// 4. 验证 URL 格式被规范化
    ///
    /// ## 预期结果
    /// - 成功获取远程 URL
    /// - SSH URL 格式被规范化为 ssh://git@host/path 格式
    ///
    /// ## 注意
    /// find_origin_remote() 会规范化 SSH URL 格式（从 git@host:path 转换为 ssh://git@host/path），
    /// 因为 git2 库不支持简写格式，所以返回的是规范化后的格式。
    #[test]
    fn test_remote_url() -> Result<()> {
        // 创建一个临时目录并初始化 Git 仓库
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // 初始化 Git 仓库（使用 git2）
        let mut repo = GitRepository::init(repo_path, None)?;

        // 添加 origin 远程（使用 git2 API）
        repo.as_inner_mut()
            .remote("origin", "git@github.com:test/repo.git")
            .map_err(|e| color_eyre::eyre::eyre!("Failed to add remote: {}", e))?;

        // 打开仓库并查找 origin 远程
        let mut repo = GitRepository::open_at(repo_path)?;
        let remote = repo.find_origin_remote()?;

        // 验证能够获取 URL
        // 注意：find_origin_remote() 会规范化 SSH URL 格式（从 git@host:path 转换为 ssh://git@host/path）
        // 因为 git2 库不支持简写格式，所以返回的是规范化后的格式
        let url = remote.url();
        assert_eq!(url, Some("ssh://git@github.com/test/repo.git"));

        Ok(())
    }

    /// 测试逃生舱方法（访问底层 git2::Remote）
    ///
    /// ## 测试目的
    /// 验证 GitRemote::as_inner() 和 as_inner_mut() 能够提供对底层 git2::Remote 的访问。
    ///
    /// ## 测试场景
    /// 1. 创建临时目录并初始化 Git 仓库
    /// 2. 添加 origin 远程仓库
    /// 3. 查找 origin 远程
    /// 4. 测试 as_inner() 和 as_inner_mut() 方法
    /// 5. 验证能够访问底层 Remote（不 panic）
    ///
    /// ## 预期结果
    /// - as_inner() 返回不可变引用
    /// - as_inner_mut() 返回可变引用
    /// - 方法执行不产生 panic
    #[test]
    fn test_as_inner() -> Result<()> {
        // 创建一个临时目录并初始化 Git 仓库
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // 初始化 Git 仓库（使用 git2）
        let mut repo = GitRepository::init(repo_path, None)?;

        // 添加 origin 远程（使用 git2 API）
        repo.as_inner_mut()
            .remote("origin", "https://github.com/test/repo.git")
            .map_err(|e| color_eyre::eyre::eyre!("Failed to add remote: {}", e))?;

        // 打开仓库并查找 origin 远程
        let mut repo = GitRepository::open_at(repo_path)?;
        let mut remote = repo.find_origin_remote()?;

        // 测试逃生舱方法
        let _inner_ref = remote.as_inner();
        let _inner_mut_ref = remote.as_inner_mut();

        // 验证能够访问底层 Remote（不应该 panic）
        // 如果上面的调用没有 panic，测试就通过了

        Ok(())
    }
}
