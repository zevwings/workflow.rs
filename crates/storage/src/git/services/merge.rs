//! Merge 业务逻辑服务
//!
//! 提供合并相关的业务逻辑实现。

use git2::BranchType;

use super::GitContext;
use domain::git::{GitError, MergeStrategy};

/// Merge 服务接口
pub trait MergeService: Send + Sync {
    /// 合并分支
    fn merge_branch(&self, source_branch: &str, strategy: MergeStrategy) -> Result<(), GitError>;

    /// 从 AnnotatedCommit 执行合并
    ///
    /// 用于 pull 等操作复用合并逻辑。
    fn merge_from_annotated(
        &self,
        annotated_commit: &git2::AnnotatedCommit,
        source_name: &str,
        strategy: MergeStrategy,
    ) -> Result<(), GitError>;

    /// 从 commit id 执行合并
    ///
    /// 用于需要避免借用冲突的场景（如 pull 操作）。
    fn merge_from_commit_id(
        &self,
        commit_id: git2::Oid,
        source_name: &str,
        strategy: MergeStrategy,
    ) -> Result<(), GitError>;

    /// 检查是否有合并冲突
    fn has_merge_conflicts(&self) -> Result<bool, GitError>;

    /// 检查分支是否已合并
    fn is_branch_merged(&self, branch: &str, base_branch: &str) -> Result<bool, GitError>;

    /// 获取合并基础
    fn merge_base(&self, branch1: &str, branch2: &str) -> Result<String, GitError>;
}

/// Merge 服务实现
pub struct MergeServiceImpl {
    ctx: GitContext,
}

impl MergeServiceImpl {
    /// 创建新的 Merge 服务实例
    pub fn new(ctx: GitContext) -> Self {
        Self { ctx }
    }

    /// 执行 fast-forward 合并（内部使用，接受 repo 引用避免死锁）
    fn do_fast_forward_with_repo(
        &self,
        repo: &git2::Repository,
        annotated_commit: &git2::AnnotatedCommit,
    ) -> Result<(), GitError> {
        let head = repo.head().map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let refname = head
            .name()
            .ok_or_else(|| GitError::OperationFailed("Invalid HEAD reference".into()))?;

        let msg = format!("Fast-Forward: {} to {}", refname, annotated_commit.id());

        let mut reference = repo
            .find_reference(refname)
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        reference
            .set_target(annotated_commit.id(), &msg)
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        repo.set_head(refname).map_err(|e| GitError::OperationFailed(e.to_string()))?;

        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        Ok(())
    }

    /// 执行普通合并（内部使用，接受 repo 引用避免死锁）
    fn do_normal_merge_with_repo(
        &self,
        repo: &git2::Repository,
        annotated_commit: &git2::AnnotatedCommit,
        source_branch: &str,
    ) -> Result<(), GitError> {
        // 执行合并
        repo.merge(&[annotated_commit], None, None)
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 检查冲突
        let index = repo.index().map_err(|e| GitError::IndexError(e.to_string()))?;

        if index.has_conflicts() {
            return Err(GitError::MergeConflict);
        }

        // 创建合并提交
        let mut index = repo.index().map_err(|e| GitError::IndexError(e.to_string()))?;
        let tree_id = index.write_tree().map_err(|e| GitError::IndexError(e.to_string()))?;
        let tree = repo.find_tree(tree_id).map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 从 repo 获取 signature，避免再次获取锁
        let signature = repo
            .signature()
            .or_else(|_| git2::Signature::now("User", "user@example.com"))
            .map_err(|e| GitError::SignatureError(e.to_string()))?;

        let head = repo.head().map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let head_commit =
            head.peel_to_commit().map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let remote_commit = repo
            .find_commit(annotated_commit.id())
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let message = format!("Merge branch '{}'", source_branch);

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &message,
            &tree,
            &[&head_commit, &remote_commit],
        )
        .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 清理状态
        repo.cleanup_state().map_err(|e| GitError::OperationFailed(e.to_string()))?;

        Ok(())
    }

    /// 执行 squash 合并（内部使用，接受 repo 引用避免死锁）
    fn do_squash_merge_with_repo(
        &self,
        repo: &git2::Repository,
        annotated_commit: &git2::AnnotatedCommit,
    ) -> Result<(), GitError> {
        // 执行合并但不创建提交
        repo.merge(&[annotated_commit], None, None)
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 检查冲突
        let index = repo.index().map_err(|e| GitError::IndexError(e.to_string()))?;

        if index.has_conflicts() {
            return Err(GitError::MergeConflict);
        }

        // 清理状态（但保留工作区更改，让用户自己提交）
        repo.cleanup_state().map_err(|e| GitError::OperationFailed(e.to_string()))?;

        Ok(())
    }
}

impl MergeService for MergeServiceImpl {
    fn merge_branch(&self, source_branch: &str, strategy: MergeStrategy) -> Result<(), GitError> {
        let repo = self.ctx.repository();

        // 查找源分支
        let source_ref = repo
            .find_branch(source_branch, BranchType::Local)
            .or_else(|_| {
                let remote_name = format!("origin/{}", source_branch);
                repo.find_branch(&remote_name, BranchType::Remote)
            })
            .map_err(|_| GitError::BranchNotFound(source_branch.to_string()))?;

        let source_commit = source_ref
            .get()
            .peel_to_commit()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let annotated_commit = repo
            .find_annotated_commit(source_commit.id())
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        self.merge_from_annotated(&annotated_commit, source_branch, strategy)
    }

    fn merge_from_annotated(
        &self,
        annotated_commit: &git2::AnnotatedCommit,
        source_name: &str,
        strategy: MergeStrategy,
    ) -> Result<(), GitError> {
        let repo = self.ctx.repository();

        // 执行合并分析
        let (analysis, _) = repo
            .merge_analysis(&[annotated_commit])
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        if analysis.is_up_to_date() {
            return Ok(());
        }

        match strategy {
            MergeStrategy::FastForwardOnly => {
                if !analysis.is_fast_forward() {
                    return Err(GitError::OperationFailed(
                        "Cannot perform fast-forward merge".into(),
                    ));
                }
                self.do_fast_forward_with_repo(&repo, annotated_commit)?;
            }
            MergeStrategy::Merge => {
                if analysis.is_fast_forward() {
                    self.do_fast_forward_with_repo(&repo, annotated_commit)?;
                } else {
                    self.do_normal_merge_with_repo(&repo, annotated_commit, source_name)?;
                }
            }
            MergeStrategy::Squash => {
                self.do_squash_merge_with_repo(&repo, annotated_commit)?;
            }
        }

        Ok(())
    }

    fn merge_from_commit_id(
        &self,
        commit_id: git2::Oid,
        source_name: &str,
        strategy: MergeStrategy,
    ) -> Result<(), GitError> {
        let repo = self.ctx.repository();
        let annotated_commit = repo
            .find_annotated_commit(commit_id)
            .map_err(|e| GitError::CommitNotFound(e.to_string()))?;

        // 执行合并分析
        let (analysis, _) = repo
            .merge_analysis(&[&annotated_commit])
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        if analysis.is_up_to_date() {
            return Ok(());
        }

        // 使用 _with_repo 版本避免再次获取锁
        match strategy {
            MergeStrategy::FastForwardOnly => {
                if !analysis.is_fast_forward() {
                    return Err(GitError::OperationFailed(
                        "Cannot perform fast-forward merge".into(),
                    ));
                }
                self.do_fast_forward_with_repo(&repo, &annotated_commit)?;
            }
            MergeStrategy::Merge => {
                if analysis.is_fast_forward() {
                    self.do_fast_forward_with_repo(&repo, &annotated_commit)?;
                } else {
                    self.do_normal_merge_with_repo(&repo, &annotated_commit, source_name)?;
                }
            }
            MergeStrategy::Squash => {
                self.do_squash_merge_with_repo(&repo, &annotated_commit)?;
            }
        }

        Ok(())
    }

    fn has_merge_conflicts(&self) -> Result<bool, GitError> {
        let repo = self.ctx.repository();

        let index = repo.index().map_err(|e| GitError::IndexError(e.to_string()))?;
        Ok(index.has_conflicts())
    }

    fn is_branch_merged(&self, branch: &str, base_branch: &str) -> Result<bool, GitError> {
        let repo = self.ctx.repository();

        // 查找分支的 commit
        let branch_ref = repo
            .find_branch(branch, BranchType::Local)
            .map_err(|_| GitError::BranchNotFound(branch.to_string()))?;
        let branch_commit = branch_ref
            .get()
            .peel_to_commit()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 查找基础分支的 commit
        let base_ref = repo
            .find_branch(base_branch, BranchType::Local)
            .map_err(|_| GitError::BranchNotFound(base_branch.to_string()))?;
        let base_commit = base_ref
            .get()
            .peel_to_commit()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 检查 branch_commit 是否是 base_commit 的祖先
        Ok(repo.graph_descendant_of(base_commit.id(), branch_commit.id()).unwrap_or(false))
    }

    fn merge_base(&self, branch1: &str, branch2: &str) -> Result<String, GitError> {
        let repo = self.ctx.repository();

        // 解析分支引用
        let obj1 = repo
            .revparse_single(branch1)
            .map_err(|_| GitError::BranchNotFound(branch1.to_string()))?;
        let obj2 = repo
            .revparse_single(branch2)
            .map_err(|_| GitError::BranchNotFound(branch2.to_string()))?;

        let commit1 =
            obj1.peel_to_commit().map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let commit2 =
            obj2.peel_to_commit().map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let merge_base_oid = repo
            .merge_base(commit1.id(), commit2.id())
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        Ok(merge_base_oid.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::testing::setup_repo_with_file;

    #[test]
    #[ignore]
    fn test_has_merge_conflicts_none() {
        let (_tmp, ctx) = setup_repo_with_file();
        let service = MergeServiceImpl::new(ctx);

        assert!(!service.has_merge_conflicts().unwrap());
    }

    #[test]
    #[ignore]
    fn test_merge_base() {
        let (_tmp, ctx) = setup_repo_with_file();

        // 创建一个分支
        let expected = {
            let repo = ctx.repository();
            let head = repo.head().unwrap();
            let commit = head.peel_to_commit().unwrap();
            repo.branch("feature", &commit, false).unwrap();
            commit.id().to_string()
        };

        let service = MergeServiceImpl::new(ctx);
        let base = service.merge_base("master", "feature").unwrap();

        // master 和 feature 指向相同的 commit
        assert_eq!(base, expected);
    }
}
