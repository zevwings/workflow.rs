//! 远程业务逻辑服务
//!
//! 提供远程相关的业务逻辑实现。

use std::sync::Arc;

use git2::{BranchType, FetchOptions, PushOptions, Repository};

use crate::git::services::context::GitContext;
use crate::git::services::hooks::{git_hooks, HookContext, HookResult, HookService};
use crate::git::services::merge::{MergeService, MergeServiceImpl};
use domain::{GitError, MergeStrategy};
use toolkit::log_debug;

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
    hook_service: Arc<dyn HookService>,
}

impl RemoteServiceImpl {
    /// 创建新的远程服务实例
    pub fn new(ctx: GitContext, hook_service: Arc<dyn HookService>) -> Self {
        Self { ctx, hook_service }
    }
}

impl RemoteService for RemoteServiceImpl {
    fn push(&self, branch_name: &str, set_upstream: bool) -> Result<(), GitError> {
        // 获取路径信息和推送相关数据
        let (repo_path, git_dir, commits_to_push) = {
            let repo = self.ctx.repository();
            let git_dir = repo.path().to_path_buf();
            let repo_path = repo
                .workdir()
                .ok_or_else(|| GitError::OperationFailed("Not a work tree".into()))?
                .to_path_buf();

            let commits_to_push = Self::get_commits_to_push(&repo, branch_name).unwrap_or_default();

            (repo_path, git_dir, commits_to_push)
        };

        // [1] pre-push hook
        let hook_context =
            HookContext::for_pre_push(repo_path, git_dir, branch_name.to_string(), commits_to_push);

        if let HookResult::Failure(msg) =
            self.hook_service.execute_hook(git_hooks::PRE_PUSH, &hook_context)?
        {
            return Err(GitError::HookFailed(format!(
                "{} hook failed: {}",
                git_hooks::PRE_PUSH,
                msg
            )));
        }

        // [2] 执行实际的 push
        let repo = self.ctx.repository();
        let mut remote =
            repo.find_remote("origin").map_err(|e| GitError::RemoteError(e.to_string()))?;

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        let callbacks = GitContext::create_callbacks();
        let mut opts = PushOptions::new();
        opts.remote_callbacks(callbacks);

        remote
            .push(&[&refspec], Some(&mut opts))
            .map_err(|e| GitError::RemoteError(e.to_string()))?;

        // 设置上游跟踪分支
        if set_upstream {
            let mut branch = repo
                .find_branch(branch_name, BranchType::Local)
                .map_err(|_| GitError::BranchNotFound(branch_name.to_string()))?;

            let upstream_name = format!("origin/{}", branch_name);
            branch
                .set_upstream(Some(&upstream_name))
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        }

        Ok(())
    }

    fn pull(&self, branch_name: &str) -> Result<(), GitError> {
        // 获取 commit id，然后释放 repo 锁，避免死锁
        let commit_id = {
            let repo = self.ctx.repository();
            let mut remote =
                repo.find_remote("origin").map_err(|e| GitError::RemoteError(e.to_string()))?;

            // Fetch
            let callbacks = GitContext::create_callbacks();
            let mut fetch_opts = FetchOptions::new();
            fetch_opts.remote_callbacks(callbacks);

            remote
                .fetch(&[branch_name], Some(&mut fetch_opts), None)
                .map_err(|e| GitError::RemoteError(e.to_string()))?;

            // 获取 FETCH_HEAD
            let fetch_head = repo
                .find_reference("FETCH_HEAD")
                .map_err(|e| GitError::InvalidReference(e.to_string()))?;

            let annotated = repo
                .reference_to_annotated_commit(&fetch_head)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;

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

impl RemoteServiceImpl {
    /// 获取将要推送的提交列表
    ///
    /// 返回本地分支相对于远程分支的新提交 SHA 列表
    fn get_commits_to_push(repo: &Repository, branch_name: &str) -> Result<Vec<String>, GitError> {
        // 获取本地分支
        let local_branch = repo
            .find_branch(branch_name, BranchType::Local)
            .map_err(|_| GitError::BranchNotFound(branch_name.to_string()))?;

        let local_commit = local_branch
            .get()
            .peel_to_commit()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 尝试获取远程分支
        let remote_branch_name = format!("origin/{}", branch_name);
        let remote_oid = repo
            .find_reference(&format!("refs/remotes/{}", remote_branch_name))
            .ok()
            .and_then(|r| r.peel_to_commit().ok())
            .map(|c| c.id());

        // 收集将要推送的提交
        let mut commits = Vec::new();
        let mut revwalk = repo.revwalk().map_err(|e| GitError::OperationFailed(e.to_string()))?;
        revwalk
            .push(local_commit.id())
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 如果有远程分支，隐藏远程分支的提交
        if let Some(oid) = remote_oid {
            if let Err(e) = revwalk.hide(oid) {
                log_debug!("Failed to hide remote oid in revwalk: {}", e);
            }
        }

        for oid in revwalk.flatten() {
            commits.push(oid.to_string());
        }

        Ok(commits)
    }
}
