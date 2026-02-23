//! Git 仓储实现
//!
//! 基于 git2 实现 GitRepository trait。

use std::sync::Arc;

use domain::{
    BlameLineInfo, BranchInfo, CommitFileChange, CommitInfo, GitError, GitRepository,
    MergeStrategy, RemoteDirection, RepoInfo, StashApplyResult, StashEntry, StashPopResult,
    TagCreateInfo, TagCreateScope, TagDeleteInfo, TagDeleteScope, WorkingTreeStatus,
};

use crate::git::services::{
    BlameService, BranchService, CommitService, DiffService, GitContext, MergeService,
    RemoteService, StashService, TagService,
};

/// Git 服务集合
///
/// 将所有 Git 子服务组合为一个结构体，简化 `GitRepositoryImpl` 的构造。
pub struct GitRepositoryServices {
    pub blame: Arc<dyn BlameService>,
    pub branch: Arc<dyn BranchService>,
    pub commit: Arc<dyn CommitService>,
    pub diff: Arc<dyn DiffService>,
    pub merge: Arc<dyn MergeService>,
    pub remote: Arc<dyn RemoteService>,
    pub tag: Arc<dyn TagService>,
    pub stash: Arc<dyn StashService>,
}

/// Git 仓储实现
pub struct GitRepositoryImpl {
    ctx: GitContext,
    services: GitRepositoryServices,
}

impl GitRepositoryImpl {
    /// 创建新的 Git 仓储实例
    pub fn new(ctx: GitContext, services: GitRepositoryServices) -> Self {
        Self { ctx, services }
    }
}

impl GitRepository for GitRepositoryImpl {
    // ========== Repo 操作 ==========

    fn get_repo_info(&self) -> RepoInfo {
        self.ctx.info()
    }

    fn get_ignore_directory_patterns(&self) -> Vec<String> {
        self.ctx.get_ignore_directory_patterns()
    }

    fn get_working_tree_diff(&self, base_branch: &str) -> Result<Option<String>, GitError> {
        self.services.diff.get_working_tree_diff(base_branch)
    }

    fn get_merge_diff(
        &self,
        branch: &str,
        target_branch: &str,
    ) -> Result<Option<String>, GitError> {
        self.services.diff.get_merge_diff(branch, target_branch)
    }

    fn get_merge_changed_files(
        &self,
        branch: &str,
        target_branch: &str,
    ) -> Result<Vec<CommitFileChange>, GitError> {
        self.services.diff.get_merge_changed_files(branch, target_branch)
    }

    fn is_formatting_only_change(
        &self,
        base_ref: &str,
        target_ref: &str,
        file_path: &str,
    ) -> Result<bool, GitError> {
        self.services.diff.is_formatting_only_change(base_ref, target_ref, file_path)
    }

    // ========== Branch 操作 ==========

    fn create_branch(&self, name: &str) -> Result<(), GitError> {
        self.services.branch.create_branch(name)
    }

    fn delete_local_branch(&self, name: &str, force: bool) -> Result<(), GitError> {
        self.services.branch.delete_local_branch(name, force)
    }

    fn delete_remote_branch(&self, name: &str) -> Result<(), GitError> {
        self.services.branch.delete_remote_branch(name)
    }

    fn rename_branch(&self, old_name: Option<&str>, new_name: &str) -> Result<(), GitError> {
        self.services.branch.rename_branch(old_name, new_name)
    }

    fn list_branches(&self, remove_prefix: bool, all: bool) -> Result<Vec<BranchInfo>, GitError> {
        self.services.branch.list_branches(remove_prefix, all)
    }

    fn checkout_branch(&self, name: &str) -> Result<(), GitError> {
        self.services.branch.checkout_branch(name)
    }

    fn get_current_branch(&self) -> Result<String, GitError> {
        self.services.branch.get_current_branch()
    }

    fn has_branch(&self, name: &str) -> Result<(bool, bool), GitError> {
        self.services.branch.has_branch(name)
    }

    fn get_default_branch(&self) -> Result<String, GitError> {
        self.services.branch.get_default_branch()
    }

    fn infer_target_branch(&self, current_branch: &str) -> Result<Option<String>, GitError> {
        self.services.branch.infer_target_branch(current_branch)
    }

    // ========== Commit 操作 ==========

    fn get_commit_info(&self, ref_or_sha: &str) -> Result<CommitInfo, GitError> {
        self.services.commit.get_commit_info(ref_or_sha)
    }

    fn get_commit_changed_files(
        &self,
        ref_or_sha: &str,
    ) -> Result<Vec<CommitFileChange>, GitError> {
        self.services.commit.get_commit_changed_files(ref_or_sha)
    }

    fn get_commit_diff(&self, ref_or_sha: &str) -> Result<Option<String>, GitError> {
        self.services.diff.get_commit_diff(ref_or_sha)
    }

    fn get_working_tree_status(&self) -> Result<WorkingTreeStatus, GitError> {
        self.services.commit.get_working_tree_status()
    }

    fn get_staged_files(&self) -> Result<Vec<CommitFileChange>, GitError> {
        self.services.commit.get_staged_files()
    }

    fn get_staged_diff(&self) -> Result<Option<String>, GitError> {
        self.services.diff.get_staged_diff()
    }

    fn add_all(&self) -> Result<(), GitError> {
        self.services.commit.add_all()
    }

    fn commit(&self, message: &str, all: bool) -> Result<String, GitError> {
        self.services.commit.commit(message, all)
    }

    // ========== Merge 操作 ==========

    fn merge_branch(&self, source_branch: &str, strategy: MergeStrategy) -> Result<(), GitError> {
        self.services.merge.merge_branch(source_branch, strategy)
    }

    fn has_merge_conflicts(&self) -> Result<bool, GitError> {
        self.services.merge.has_merge_conflicts()
    }

    fn is_branch_merged(&self, branch: &str, base_branch: &str) -> Result<bool, GitError> {
        self.services.merge.is_branch_merged(branch, base_branch)
    }

    fn merge_base(&self, branch1: &str, branch2: &str) -> Result<String, GitError> {
        self.services.merge.merge_base(branch1, branch2)
    }

    fn commits_to_merge(
        &self,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<Vec<String>, GitError> {
        self.services.merge.commits_to_merge(source_branch, target_branch)
    }

    // ========== Rebase 操作 ==========

    fn rebase_onto(&self, _target_branch: &str) -> Result<(), GitError> {
        Err(GitError::OperationFailed(
            "rebase_onto is not implemented yet".to_string(),
        ))
    }

    fn rebase_onto_with_upstream(
        &self,
        _newbase: &str,
        _upstream: &str,
        _branch: &str,
    ) -> Result<(), GitError> {
        Err(GitError::OperationFailed(
            "rebase_onto_with_upstream is not implemented yet".to_string(),
        ))
    }

    // ========== Remote 操作 ==========

    fn push(&self, branch_name: &str, set_upstream: bool) -> Result<(), GitError> {
        self.services.remote.push(branch_name, set_upstream)
    }

    fn pull(&self, branch_name: &str) -> Result<(), GitError> {
        self.services.remote.pull(branch_name)
    }

    fn is_commit_in_remote_branch(&self, branch: &str, commit_sha: &str) -> Result<bool, GitError> {
        self.services.remote.is_commit_in_remote_branch(branch, commit_sha)
    }

    fn is_remote_available(&self) -> Result<Vec<RemoteDirection>, GitError> {
        self.services.remote.is_remote_available()
    }

    // ========== Stash 操作 ==========

    fn stash_push(&self, message: Option<&str>) -> Result<usize, GitError> {
        self.services.stash.stash_push(message)
    }

    fn stash_pop(&self, index: usize) -> Result<StashPopResult, GitError> {
        self.services.stash.stash_pop(index)
    }

    fn stash_apply(&self, index: usize) -> Result<StashApplyResult, GitError> {
        self.services.stash.stash_apply(index)
    }

    fn stash_list(&self) -> Result<Vec<StashEntry>, GitError> {
        self.services.stash.stash_list()
    }

    fn stash_drop(&self, index: usize) -> Result<(), GitError> {
        self.services.stash.stash_drop(index)
    }

    // ========== Tag 操作 ==========

    fn create_tag(
        &self,
        name: &str,
        target: Option<&str>,
        message: Option<&str>,
        scope: TagCreateScope,
        force: bool,
    ) -> Result<TagCreateInfo, GitError> {
        self.services.tag.create_tag(name, target, message, scope, force)
    }

    fn delete_tag(
        &self,
        name: &str,
        scope: TagDeleteScope,
        force: bool,
    ) -> Result<TagDeleteInfo, GitError> {
        self.services.tag.delete_tag(name, scope, force)
    }

    fn delete_tags_by_pattern(
        &self,
        pattern: &str,
        scope: TagDeleteScope,
        force: bool,
    ) -> Result<Vec<TagDeleteInfo>, GitError> {
        self.services.tag.delete_tags_by_pattern(pattern, scope, force)
    }

    fn list_tags(&self, include_remote: bool) -> Result<Vec<String>, GitError> {
        self.services.tag.list_tags(include_remote)
    }

    fn has_tag(&self, name: &str) -> Result<(bool, bool), GitError> {
        self.services.tag.has_tag(name)
    }

    fn preview_delete(
        &self,
        name: Option<&str>,
        pattern: Option<&str>,
        scope: TagDeleteScope,
    ) -> Result<Vec<TagDeleteInfo>, GitError> {
        self.services.tag.preview_delete(name, pattern, scope)
    }

    // ========== Blame 操作 ==========

    fn get_file_blame(
        &self,
        file_path: &str,
        revision: Option<&str>,
    ) -> Result<Vec<BlameLineInfo>, GitError> {
        self.services.blame.get_file_blame(file_path, revision)
    }

    fn get_file_blame_range(
        &self,
        file_path: &str,
        start_line: usize,
        end_line: usize,
        revision: Option<&str>,
    ) -> Result<Vec<BlameLineInfo>, GitError> {
        self.services
            .blame
            .get_file_blame_range(file_path, start_line, end_line, revision)
    }
}
