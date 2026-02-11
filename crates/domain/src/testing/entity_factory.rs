//! 实体测试数据构建器
//!
//! 提供领域实体的 Builder，便于在测试中创建具有默认值或自定义字段的实体。
//!
//! # 示例
//!
//! ```ignore
//! use domain::testing::TestEntityFactory;
//!
//! let branch = TestEntityFactory::branch_info()
//!     .with_name("feature/new-feature")
//!     .as_current()
//!     .with_upstream("origin/feature/new-feature")
//!     .build();
//! ```

use crate::{
    BranchInfo, CodePlatform, CommitInfo, PullRequestInfo, PullRequestStatus, RemoteInfo, RepoInfo,
    StashEntry,
};

/// 领域实体测试数据工厂
///
/// 提供各实体的构建器入口，用于在测试中快速创建领域对象。
pub struct TestEntityFactory;

impl TestEntityFactory {
    pub fn branch_info() -> BranchInfoBuilder {
        BranchInfoBuilder::default()
    }

    pub fn commit_info() -> CommitInfoBuilder {
        CommitInfoBuilder::default()
    }

    pub fn pull_request_info() -> PullRequestInfoBuilder {
        PullRequestInfoBuilder::default()
    }

    pub fn stash_entry() -> StashEntryBuilder {
        StashEntryBuilder::default()
    }

    pub fn remote_info() -> RemoteInfoBuilder {
        RemoteInfoBuilder::default()
    }

    pub fn repo_info() -> RepoInfoBuilder {
        RepoInfoBuilder::default()
    }
}

// =============================================================================
// BranchInfo 构建器
// =============================================================================

/// 分支信息构建器
#[derive(Default)]
pub struct BranchInfoBuilder {
    name: Option<String>,
    display_name: Option<String>,
    is_remote: bool,
    is_current: bool,
    commit_sha: Option<String>,
    commit_message: Option<String>,
    upstream: Option<String>,
}

impl BranchInfoBuilder {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        if self.display_name.is_none() {
            self.display_name = Some(name.clone());
        }
        self.name = Some(name);
        self
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn as_current(mut self) -> Self {
        self.is_current = true;
        self
    }

    pub fn as_remote(mut self) -> Self {
        self.is_remote = true;
        self
    }

    pub fn with_upstream(mut self, upstream: impl Into<String>) -> Self {
        self.upstream = Some(upstream.into());
        self
    }

    pub fn with_commit_sha(mut self, sha: impl Into<String>) -> Self {
        self.commit_sha = Some(sha.into());
        self
    }

    pub fn with_commit_message(mut self, message: impl Into<String>) -> Self {
        self.commit_message = Some(message.into());
        self
    }

    pub fn build(self) -> BranchInfo {
        let name = self.name.unwrap_or_else(|| "test-branch".to_string());
        let display_name = self.display_name.unwrap_or_else(|| name.clone());
        BranchInfo {
            name,
            display_name,
            is_remote: self.is_remote,
            is_current: self.is_current,
            commit_sha: self.commit_sha,
            commit_message: self.commit_message,
            upstream: self.upstream,
        }
    }
}

// =============================================================================
// CommitInfo 构建器
// =============================================================================

/// 提交信息构建器
#[derive(Default)]
pub struct CommitInfoBuilder {
    sha: Option<String>,
    message: Option<String>,
    summary: Option<String>,
    author_name: Option<String>,
    author_email: Option<String>,
    author_time: Option<i64>,
    committer_name: Option<String>,
    committer_email: Option<String>,
    committer_time: Option<i64>,
    parents: Option<Vec<String>>,
}

impl CommitInfoBuilder {
    pub fn with_sha(mut self, sha: impl Into<String>) -> Self {
        self.sha = Some(sha.into());
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        let message = message.into();
        if self.summary.is_none() {
            self.summary = Some(message.lines().next().unwrap_or("").to_string());
        }
        self.message = Some(message);
        self
    }

    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    pub fn with_author_name(mut self, name: impl Into<String>) -> Self {
        self.author_name = Some(name.into());
        self
    }

    pub fn with_author_email(mut self, email: impl Into<String>) -> Self {
        self.author_email = Some(email.into());
        self
    }

    pub fn with_author_time(mut self, time: i64) -> Self {
        self.author_time = Some(time);
        self
    }

    pub fn with_committer_name(mut self, name: impl Into<String>) -> Self {
        self.committer_name = Some(name.into());
        self
    }

    pub fn with_committer_email(mut self, email: impl Into<String>) -> Self {
        self.committer_email = Some(email.into());
        self
    }

    pub fn with_committer_time(mut self, time: i64) -> Self {
        self.committer_time = Some(time);
        self
    }

    pub fn with_parents(mut self, parents: Vec<String>) -> Self {
        self.parents = Some(parents);
        self
    }

    pub fn build(self) -> CommitInfo {
        let message = self.message.unwrap_or_else(|| "test commit message".to_string());
        let summary =
            self.summary.unwrap_or_else(|| message.lines().next().unwrap_or("").to_string());
        CommitInfo {
            sha: self.sha.unwrap_or_else(|| "abc123def456".to_string()),
            message,
            summary,
            author_name: self.author_name.unwrap_or_else(|| "Test Author".to_string()),
            author_email: self.author_email.unwrap_or_else(|| "author@test.local".to_string()),
            author_time: self.author_time.unwrap_or(0),
            committer_name: self.committer_name.unwrap_or_else(|| "Test Committer".to_string()),
            committer_email: self
                .committer_email
                .unwrap_or_else(|| "committer@test.local".to_string()),
            committer_time: self.committer_time.unwrap_or(0),
            parents: self.parents.unwrap_or_default(),
        }
    }
}

// =============================================================================
// PullRequestInfo 构建器
// =============================================================================

/// Pull Request 信息构建器
#[derive(Default)]
pub struct PullRequestInfoBuilder {
    id: Option<String>,
    title: Option<String>,
    body: Option<String>,
    status: Option<PullRequestStatus>,
    source_branch: Option<String>,
    target_branch: Option<String>,
}

impl PullRequestInfoBuilder {
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn with_status(mut self, status: PullRequestStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_source_branch(mut self, branch: impl Into<String>) -> Self {
        self.source_branch = Some(branch.into());
        self
    }

    pub fn with_target_branch(mut self, branch: impl Into<String>) -> Self {
        self.target_branch = Some(branch.into());
        self
    }

    pub fn build(self) -> PullRequestInfo {
        PullRequestInfo {
            id: self.id.unwrap_or_else(|| "1".to_string()),
            title: self.title.unwrap_or_else(|| "Test PR".to_string()),
            body: self.body.unwrap_or_else(|| "Test body".to_string()),
            status: self.status.unwrap_or_else(|| PullRequestStatus {
                state: "open".to_string(),
                merged: false,
                merged_at: None,
            }),
            source_branch: self.source_branch.unwrap_or_else(|| "feature/test".to_string()),
            target_branch: self.target_branch.unwrap_or_else(|| "main".to_string()),
        }
    }
}

// =============================================================================
// StashEntry 构建器
// =============================================================================

/// Stash 条目构建器
#[derive(Default)]
pub struct StashEntryBuilder {
    index: Option<usize>,
    branch: Option<String>,
    message: Option<String>,
    commit_hash: Option<String>,
    timestamp: Option<chrono::DateTime<chrono::Local>>,
}

impl StashEntryBuilder {
    pub fn with_index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn with_commit_hash(mut self, hash: impl Into<String>) -> Self {
        self.commit_hash = Some(hash.into());
        self
    }

    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Local>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn build(self) -> StashEntry {
        StashEntry {
            index: self.index.unwrap_or(0),
            branch: self.branch.unwrap_or_else(|| "main".to_string()),
            message: self.message.unwrap_or_else(|| "WIP".to_string()),
            commit_hash: self.commit_hash.unwrap_or_else(|| "abc123".to_string()),
            timestamp: self.timestamp,
        }
    }
}

// =============================================================================
// RemoteInfo 构建器
// =============================================================================

/// 远程信息构建器
#[derive(Default)]
pub struct RemoteInfoBuilder {
    name: Option<String>,
    url: Option<String>,
    push_url: Option<String>,
}

impl RemoteInfoBuilder {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn with_push_url(mut self, push_url: impl Into<String>) -> Self {
        self.push_url = Some(push_url.into());
        self
    }

    pub fn build(self) -> RemoteInfo {
        RemoteInfo {
            name: self.name.unwrap_or_else(|| "origin".to_string()),
            url: self.url.unwrap_or_else(|| "https://github.com/owner/repo.git".to_string()),
            push_url: self.push_url,
        }
    }
}

// =============================================================================
// RepoInfo 构建器
// =============================================================================

/// 仓库信息构建器
#[derive(Default)]
pub struct RepoInfoBuilder {
    is_valid: Option<bool>,
    kind: Option<CodePlatform>,
    origin_url: Option<String>,
    directory: Option<String>,
    name: Option<String>,
    owner: Option<String>,
}

impl RepoInfoBuilder {
    pub fn valid(mut self, valid: bool) -> Self {
        self.is_valid = Some(valid);
        self
    }

    pub fn with_kind(mut self, kind: CodePlatform) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn with_origin_url(mut self, url: impl Into<String>) -> Self {
        self.origin_url = Some(url.into());
        self
    }

    pub fn with_directory(mut self, dir: impl Into<String>) -> Self {
        self.directory = Some(dir.into());
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = Some(owner.into());
        self
    }

    pub fn build(self) -> RepoInfo {
        RepoInfo {
            is_valid: self.is_valid.unwrap_or(true),
            kind: self.kind,
            origin_url: self.origin_url,
            directory: self.directory,
            name: self.name,
            owner: self.owner,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_info_builder_default() {
        let branch = TestEntityFactory::branch_info().build();
        assert_eq!(branch.name, "test-branch");
        assert_eq!(branch.display_name, "test-branch");
        assert!(!branch.is_current);
        assert!(branch.upstream.is_none());
    }

    #[test]
    fn test_branch_info_builder_custom() {
        let branch = TestEntityFactory::branch_info()
            .with_name("feature/new-feature")
            .as_current()
            .with_upstream("origin/feature/new-feature")
            .build();
        assert_eq!(branch.name, "feature/new-feature");
        assert!(branch.is_current);
        assert_eq!(
            branch.upstream.as_deref(),
            Some("origin/feature/new-feature")
        );
    }

    #[test]
    fn test_commit_info_builder_default() {
        let commit = TestEntityFactory::commit_info().build();
        assert_eq!(commit.sha, "abc123def456");
        assert_eq!(commit.message, "test commit message");
        assert_eq!(commit.author_name, "Test Author");
    }

    #[test]
    fn test_pull_request_info_builder_default() {
        let pr = TestEntityFactory::pull_request_info().build();
        assert_eq!(pr.id, "1");
        assert_eq!(pr.title, "Test PR");
        assert_eq!(pr.status.state, "open");
        assert!(!pr.status.merged);
    }

    #[test]
    fn test_stash_entry_builder_default() {
        let stash = TestEntityFactory::stash_entry().build();
        assert_eq!(stash.index, 0);
        assert_eq!(stash.branch, "main");
        assert_eq!(stash.message, "WIP");
    }

    #[test]
    fn test_remote_info_builder_default() {
        let remote = TestEntityFactory::remote_info().build();
        assert_eq!(remote.name, "origin");
        assert!(remote.url.contains("github.com"));
    }

    #[test]
    fn test_repo_info_builder_default() {
        let repo = TestEntityFactory::repo_info().build();
        assert!(repo.is_valid);
        assert!(repo.kind.is_none());
    }
}
