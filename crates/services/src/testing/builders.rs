//! 测试数据构建器
//!
//! 提供链式 API 预置 Mock 服务的返回数据，简化测试编写。

use domain::CommitSummaryAnalysis;

use crate::testing::{MockBranchService, MockCommitMessageService, MockPullRequestService};

// ==================== BranchServiceTestData ====================

/// 分支服务测试数据构建器
///
/// 用于快速构建带预置返回值的 MockBranchService。
pub struct BranchServiceTestData {
    service: MockBranchService,
}

impl BranchServiceTestData {
    /// 创建新的测试数据构建器
    pub fn new() -> Self {
        Self {
            service: MockBranchService::new(),
        }
    }

    /// 设置默认返回的分支名（当未添加 response 时使用）
    pub fn with_default_name(self, name: impl Into<String>) -> Self {
        self.service.set_default_name(name);
        self
    }

    /// 添加一次 `generate_branch_name` 调用的返回值（按调用顺序）
    pub fn with_response(self, name: impl Into<String>) -> Self {
        self.service.add_response(name);
        self
    }

    /// 添加多次返回值
    pub fn with_responses(self, names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for n in names {
            self.service.add_response(n);
        }
        self
    }

    /// 构建并返回 Mock 服务
    pub fn build(self) -> MockBranchService {
        self.service
    }
}

impl Default for BranchServiceTestData {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== CommitMessageServiceTestData ====================

/// Commit Message 服务测试数据构建器
///
/// 用于快速构建带预置返回值的 MockCommitMessageService。
pub struct CommitMessageServiceTestData {
    service: MockCommitMessageService,
}

impl CommitMessageServiceTestData {
    /// 创建新的测试数据构建器
    pub fn new() -> Self {
        Self {
            service: MockCommitMessageService::new(),
        }
    }

    /// 添加一次成功返回值（用于 generate_for_staged 或 generate_for_commit）
    pub fn with_analysis(self, analysis: CommitSummaryAnalysis) -> Self {
        self.service.add_response(analysis);
        self
    }

    /// 添加一次失败返回值
    pub fn with_error(self, msg: impl Into<String>) -> Self {
        self.service.add_error(msg);
        self
    }

    /// 构建并返回 Mock 服务
    pub fn build(self) -> MockCommitMessageService {
        self.service
    }
}

impl Default for CommitMessageServiceTestData {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== PullRequestServiceTestData ====================

/// Pull Request 服务测试数据构建器
///
/// 用于快速构建带预置 PR 数据的 MockPullRequestService。
pub struct PullRequestServiceTestData {
    service: MockPullRequestService,
}

impl PullRequestServiceTestData {
    /// 创建新的测试数据构建器
    pub fn new() -> Self {
        Self {
            service: MockPullRequestService::new(),
        }
    }

    /// 添加一个 PR
    pub fn with_pr(
        self,
        id: impl Into<String>,
        title: impl Into<String>,
        body: impl Into<String>,
        state: impl Into<String>,
        merged: bool,
        source_branch: impl Into<String>,
        target_branch: impl Into<String>,
    ) -> Self {
        self.service
            .add_pr(id, title, body, state, merged, source_branch, target_branch);
        self
    }

    /// 添加一个开放的 PR（常用快捷方法）
    pub fn with_open_pr(
        self,
        id: impl Into<String>,
        title: impl Into<String>,
        source_branch: impl Into<String>,
        target_branch: impl Into<String>,
    ) -> Self {
        self.service.add_pr(id, title, "", "open", false, source_branch, target_branch);
        self
    }

    /// 构建并返回 Mock 服务
    pub fn build(self) -> MockPullRequestService {
        self.service
    }
}

impl Default for PullRequestServiceTestData {
    fn default() -> Self {
        Self::new()
    }
}
