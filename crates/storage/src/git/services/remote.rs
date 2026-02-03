//! 远程业务逻辑服务
//!
//! 提供远程相关的业务逻辑实现。

use super::{GitContext, MergeService, MergeServiceImpl};
use domain::git::{GitError, MergeStrategy};
use git2::PushOptions;

/// 远程服务接口
pub trait RemoteService: Send + Sync {
    /// 推送分支到远程
    fn push(&self, branch_name: &str, set_upstream: bool) -> Result<(), GitError>;

    /// 从远程拉取分支
    fn pull(&self, branch_name: &str) -> Result<(), GitError>;

    /// 检查提交是否在远程分支中
    fn is_commit_in_remote_branch(&self, branch: &str, commit_sha: &str) -> Result<bool, GitError>;
}

/// 远程服务实现
pub struct RemoteServiceImpl {
    ctx: GitContext,
}

impl RemoteServiceImpl {
    /// 创建新的远程服务实例
    pub fn new(ctx: GitContext) -> Self {
        Self { ctx }
    }
}

impl RemoteService for RemoteServiceImpl {
    fn push(&self, branch_name: &str, set_upstream: bool) -> Result<(), GitError> {
        let repo = self.ctx.repository();
        let mut remote = repo
            .find_remote("origin")
            .map_err(|e| GitError::RemoteError(format!("找不到远程 'origin': {}", e)))?;

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        let callbacks = GitContext::create_callbacks();
        let mut opts = PushOptions::new();
        opts.remote_callbacks(callbacks);

        remote
            .push(&[&refspec], Some(&mut opts))
            .map_err(|e| GitError::RemoteError(format!("推送失败: {}", e)))?;

        // 设置上游跟踪分支
        if set_upstream {
            let mut branch =
                repo.find_branch(branch_name, git2::BranchType::Local).map_err(|e| {
                    GitError::OperationFailed(format!("找不到分支 '{}': {}", branch_name, e))
                })?;

            let upstream_name = format!("origin/{}", branch_name);
            branch
                .set_upstream(Some(&upstream_name))
                .map_err(|e| GitError::OperationFailed(format!("设置上游分支失败: {}", e)))?;
        }

        Ok(())
    }

    fn pull(&self, branch_name: &str) -> Result<(), GitError> {
        // 获取 commit id，然后释放 repo 锁，避免死锁
        let commit_id = {
            let repo = self.ctx.repository();
            let mut remote = repo
                .find_remote("origin")
                .map_err(|e| GitError::RemoteError(format!("找不到远程 'origin': {}", e)))?;

            // Fetch
            let callbacks = GitContext::create_callbacks();
            let mut fetch_opts = git2::FetchOptions::new();
            fetch_opts.remote_callbacks(callbacks);

            remote
                .fetch(&[branch_name], Some(&mut fetch_opts), None)
                .map_err(|e| GitError::RemoteError(format!("拉取失败: {}", e)))?;

            // 获取 FETCH_HEAD
            let fetch_head = repo
                .find_reference("FETCH_HEAD")
                .map_err(|e| GitError::OperationFailed(format!("找不到 FETCH_HEAD: {}", e)))?;

            let annotated = repo
                .reference_to_annotated_commit(&fetch_head)
                .map_err(|e| GitError::OperationFailed(format!("无法获取注释提交: {}", e)))?;

            annotated.id()
            // repo 的 MutexGuard 在这里释放
        };

        // 使用 MergeService 执行合并（现在可以安全地获取锁了）
        let merge_service = MergeServiceImpl::new(self.ctx.clone());
        merge_service.merge_from_commit_id(commit_id, branch_name, MergeStrategy::Merge)
    }

    fn is_commit_in_remote_branch(&self, branch: &str, commit_sha: &str) -> Result<bool, GitError> {
        let repo = self.ctx.repository();
        let remote_branch = format!("origin/{}", branch);

        // 尝试找到远程分支
        let remote_ref = match repo.find_reference(&format!("refs/remotes/{}", remote_branch)) {
            Ok(r) => r,
            Err(_) => return Ok(false), // 远程分支不存在
        };

        let remote_commit = remote_ref
            .peel_to_commit()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 解析目标提交
        let target_obj = repo
            .revparse_single(commit_sha)
            .map_err(|_| GitError::CommitNotFound(commit_sha.to_string()))?;
        let target_commit = target_obj
            .peel_to_commit()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 检查是否为祖先
        Ok(repo
            .graph_descendant_of(remote_commit.id(), target_commit.id())
            .unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::testing::setup_repo;

    #[test]
    fn test_is_commit_in_remote_branch_no_remote() {
        let (_tmp, ctx) = setup_repo();
        let sha = {
            let repo = ctx.repository();
            let commit_id = {
                let head = repo.head().unwrap();
                head.peel_to_commit().unwrap().id()
            };
            drop(repo);
            commit_id.to_string()
        };

        let service = RemoteServiceImpl::new(ctx);

        // 没有远程分支，应该返回 false
        let result = service.is_commit_in_remote_branch("main", &sha).unwrap();
        assert!(!result);
    }
}
