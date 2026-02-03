//! Git 仓储实现
//!
//! 基于 git2 实现 GitRepository trait。

use std::sync::Arc;

use domain::git::{
    BlameLineInfo, CommitInfo, GitError, GitRepository, MergeStrategy, RepoInfo, StashApplyResult,
    StashEntry, StashPopResult, TagCreateInfo, TagCreateScope, TagDeleteInfo, TagDeleteScope,
    WorkingTreeStatus,
};

use crate::git::services::{
    BlameService, BranchService, CommitService, DiffService, GitContext, MergeService,
    RemoteService, StashService, TagService,
};

/// Git 仓储实现
pub struct GitRepositoryImpl {
    ctx: GitContext,
    blame_service: Arc<dyn BlameService>,
    branch_service: Arc<dyn BranchService>,
    commit_service: Arc<dyn CommitService>,
    diff_service: Arc<dyn DiffService>,
    merge_service: Arc<dyn MergeService>,
    remote_service: Arc<dyn RemoteService>,
    tag_service: Arc<dyn TagService>,
    stash_service: Arc<dyn StashService>,
}

impl GitRepositoryImpl {
    /// 创建新的 Git 仓储实例
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ctx: GitContext,
        blame_service: Arc<dyn BlameService>,
        branch_service: Arc<dyn BranchService>,
        commit_service: Arc<dyn CommitService>,
        diff_service: Arc<dyn DiffService>,
        merge_service: Arc<dyn MergeService>,
        remote_service: Arc<dyn RemoteService>,
        tag_service: Arc<dyn TagService>,
        stash_service: Arc<dyn StashService>,
    ) -> Self {
        Self {
            ctx,
            blame_service,
            branch_service,
            commit_service,
            diff_service,
            merge_service,
            remote_service,
            tag_service,
            stash_service,
        }
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
        self.diff_service.get_working_tree_diff(base_branch)
    }

    // ========== Branch 操作 ==========

    fn create_branch(&self, name: &str) -> Result<(), GitError> {
        self.branch_service.create_branch(name)
    }

    fn delete_branch(&self, name: &str, force: bool) -> Result<(), GitError> {
        self.branch_service.delete_branch(name, force)
    }

    fn rename_branch(&self, old_name: Option<&str>, new_name: &str) -> Result<(), GitError> {
        self.branch_service.rename_branch(old_name, new_name)
    }

    fn list_branches(
        &self,
        remove_prefix: bool,
        all: bool,
    ) -> Result<Vec<domain::BranchInfo>, GitError> {
        self.branch_service.list_branches(remove_prefix, all)
    }

    fn checkout_branch(&self, name: &str) -> Result<(), GitError> {
        self.branch_service.checkout_branch(name)
    }

    fn get_current_branch(&self) -> Result<String, GitError> {
        self.branch_service.get_current_branch()
    }

    fn has_branch(&self, name: &str) -> Result<(bool, bool), GitError> {
        self.branch_service.has_branch(name)
    }

    fn get_default_branch(&self) -> Result<String, GitError> {
        self.branch_service.get_default_branch()
    }

    fn infer_target_branch(&self, current_branch: &str) -> Result<Option<String>, GitError> {
        self.branch_service.infer_target_branch(current_branch)
    }

    // ========== Commit 操作 ==========

    fn get_commit_info(&self, ref_or_sha: &str) -> Result<CommitInfo, GitError> {
        self.commit_service.get_commit_info(ref_or_sha)
    }

    fn get_working_tree_status(&self) -> Result<WorkingTreeStatus, GitError> {
        self.commit_service.get_working_tree_status()
    }

    fn amend_commit(
        &self,
        message: Option<&str>,
        no_edit: bool,
        no_verify: bool,
    ) -> Result<String, GitError> {
        self.commit_service.amend_commit(message, no_edit, no_verify)
    }

    fn commit(&self, message: &str, all: bool) -> Result<String, GitError> {
        self.commit_service.commit(message, all)
    }

    // ========== Merge 操作 ==========

    fn merge_branch(&self, source_branch: &str, strategy: MergeStrategy) -> Result<(), GitError> {
        self.merge_service.merge_branch(source_branch, strategy)
    }

    fn has_merge_conflicts(&self) -> Result<bool, GitError> {
        self.merge_service.has_merge_conflicts()
    }

    fn is_branch_merged(&self, branch: &str, base_branch: &str) -> Result<bool, GitError> {
        self.merge_service.is_branch_merged(branch, base_branch)
    }

    fn merge_base(&self, branch1: &str, branch2: &str) -> Result<String, GitError> {
        self.merge_service.merge_base(branch1, branch2)
    }

    // ========== Rebase 操作 ==========

    fn rebase_onto(&self, _target_branch: &str) -> Result<(), GitError> {
        todo!("rebase_onto 待实现")
    }

    fn rebase_onto_with_upstream(
        &self,
        _newbase: &str,
        _upstream: &str,
        _branch: &str,
    ) -> Result<(), GitError> {
        todo!("rebase_onto_with_upstream 待实现")
    }

    // ========== Remote 操作 ==========

    fn push(&self, branch_name: &str, set_upstream: bool) -> Result<(), GitError> {
        self.remote_service.push(branch_name, set_upstream)
    }

    fn pull(&self, branch_name: &str) -> Result<(), GitError> {
        self.remote_service.pull(branch_name)
    }

    fn is_commit_in_remote_branch(&self, branch: &str, commit_sha: &str) -> Result<bool, GitError> {
        self.remote_service.is_commit_in_remote_branch(branch, commit_sha)
    }

    // ========== Stash 操作 ==========

    fn stash_push(&self, message: Option<&str>) -> Result<usize, GitError> {
        self.stash_service.stash_push(message)
    }

    fn stash_pop(&self, index: usize) -> Result<StashPopResult, GitError> {
        self.stash_service.stash_pop(index)
    }

    fn stash_apply(&self, index: usize) -> Result<StashApplyResult, GitError> {
        self.stash_service.stash_apply(index)
    }

    fn stash_list(&self) -> Result<Vec<StashEntry>, GitError> {
        self.stash_service.stash_list()
    }

    fn stash_drop(&self, index: usize) -> Result<(), GitError> {
        self.stash_service.stash_drop(index)
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
        self.tag_service.create_tag(name, target, message, scope, force)
    }

    fn delete_tag(
        &self,
        name: &str,
        scope: TagDeleteScope,
        force: bool,
    ) -> Result<TagDeleteInfo, GitError> {
        self.tag_service.delete_tag(name, scope, force)
    }

    fn delete_tags_by_pattern(
        &self,
        pattern: &str,
        scope: TagDeleteScope,
        force: bool,
    ) -> Result<Vec<TagDeleteInfo>, GitError> {
        self.tag_service.delete_tags_by_pattern(pattern, scope, force)
    }

    fn list_tags(&self, include_remote: bool) -> Result<Vec<String>, GitError> {
        self.tag_service.list_tags(include_remote)
    }

    fn has_tag(&self, name: &str) -> Result<(bool, bool), GitError> {
        self.tag_service.has_tag(name)
    }

    fn preview_delete(
        &self,
        name: Option<&str>,
        pattern: Option<&str>,
        scope: TagDeleteScope,
    ) -> Result<Vec<TagDeleteInfo>, GitError> {
        self.tag_service.preview_delete(name, pattern, scope)
    }

    // ========== Blame 操作 ==========

    fn get_file_blame(
        &self,
        file_path: &str,
        revision: Option<&str>,
    ) -> Result<Vec<BlameLineInfo>, GitError> {
        self.blame_service.get_file_blame(file_path, revision)
    }

    fn get_file_blame_range(
        &self,
        file_path: &str,
        start_line: usize,
        end_line: usize,
        revision: Option<&str>,
    ) -> Result<Vec<BlameLineInfo>, GitError> {
        self.blame_service
            .get_file_blame_range(file_path, start_line, end_line, revision)
    }
}
