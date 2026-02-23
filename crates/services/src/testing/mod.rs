//! 服务层测试辅助工具
//!
//! 提供业务服务的 Mock 与测试数据构建器，供 app 集成测试使用，无需真实 LLM 或 Git。
//!
//! # 功能启用
//!
//! 本 crate 内测试自动可用；其他 crate 需在 `Cargo.toml` 启用：
//!
//! ```toml
//! [dev-dependencies]
//! services = { path = "../services", features = ["testing"] }
//! ```
//!
//! # 使用示例
//!
//! ## Mock 服务
//!
//! ```ignore
//! use services::testing::{MockBranchService, MockPullRequestService};
//! use domain::BranchService;
//!
//! let branch = MockBranchService::new();
//! branch.add_response("feature/my-feature");
//! let name = branch.generate_branch_name(Some("title"), &[]).unwrap();
//!
//! let pr = MockPullRequestService::new();
//! pr.add_pr("1", "Title", "Body", "open", false, "feat", "main");
//! let list = pr.list_pull_requests(None, None).unwrap();
//! ```
//!
//! ## 构建器
//!
//! ```ignore
//! use services::testing::{BranchServiceTestData, PullRequestServiceTestData};
//!
//! let branch = BranchServiceTestData::new()
//!     .with_response("feature/my-feature")
//!     .build();
//!
//! let pr = PullRequestServiceTestData::new()
//!     .with_open_pr("1", "Title", "feat", "main")
//!     .build();
//! ```
//!
//! # 注意事项
//!
//! - 其他 crate 使用须启用 `testing` feature；本模块不参与生产构建。

pub mod builders;
pub mod mock_services;

pub use builders::{
    BranchServiceTestData, CommitMessageServiceTestData, PullRequestServiceTestData,
};
pub use mock_services::{MockBranchService, MockCommitMessageService, MockPullRequestService};
