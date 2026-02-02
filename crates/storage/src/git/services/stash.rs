//! Stash 业务逻辑服务
//!
//! 提供 Git stash 相关的业务逻辑实现。

use super::GitContext;
use domain::git::GitError;

/// Stash 服务接口
pub trait StashService: Send + Sync {
    /// 创建 stash
    ///
    /// # 参数
    /// - `message`: 可选的 stash 消息
    ///
    /// # 返回
    /// 返回创建的 stash 的索引（0 表示最新的 stash）
    fn stash_push(&self, message: Option<&str>) -> Result<usize, GitError>;

    /// 应用并删除 stash
    ///
    /// # 参数
    /// - `index`: stash 索引（0 表示最新的 stash）
    ///
    /// # 返回
    /// 成功返回 Ok(())
    fn stash_pop(&self, index: usize) -> Result<(), GitError>;

    /// 应用 stash（不删除）
    ///
    /// # 参数
    /// - `index`: stash 索引（0 表示最新的 stash）
    ///
    /// # 返回
    /// 成功返回 Ok(())
    fn stash_apply(&self, index: usize) -> Result<(), GitError>;
}

/// Stash 服务实现
pub struct StashServiceImpl {
    ctx: GitContext,
}

impl StashServiceImpl {
    /// 创建新的 Stash 服务实例
    pub fn new(ctx: GitContext) -> Self {
        Self { ctx }
    }
}

impl StashService for StashServiceImpl {
    fn stash_push(&self, message: Option<&str>) -> Result<usize, GitError> {
        let mut repo = self.ctx.repository_mut();
        let signature = repo.signature().map_err(|e| {
            GitError::OperationFailed(format!("无法获取 Git 签名: {}", e))
        })?;

        let stash_message = message.unwrap_or("Stashed changes");
        let flags = git2::StashFlags::INCLUDE_UNTRACKED;

        repo.stash_save(&signature, stash_message, Some(flags))
            .map_err(|e| GitError::OperationFailed(format!("创建 stash 失败: {}", e)))?;

        // 返回 0，表示最新的 stash
        Ok(0)
    }

    fn stash_pop(&self, index: usize) -> Result<(), GitError> {
        let mut repo = self.ctx.repository_mut();
        let mut options = git2::StashApplyOptions::default();

        repo.stash_apply(index, Some(&mut options))
            .map_err(|e| GitError::OperationFailed(format!("应用 stash 失败: {}", e)))?;

        // 应用成功后删除 stash
        repo.stash_drop(index)
            .map_err(|e| GitError::OperationFailed(format!("删除 stash 失败: {}", e)))?;

        Ok(())
    }

    fn stash_apply(&self, index: usize) -> Result<(), GitError> {
        let mut repo = self.ctx.repository_mut();
        let mut options = git2::StashApplyOptions::default();

        repo.stash_apply(index, Some(&mut options))
            .map_err(|e| GitError::OperationFailed(format!("应用 stash 失败: {}", e)))?;

        Ok(())
    }
}
