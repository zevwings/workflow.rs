//! 领域层测试辅助工具
//!
//! 在启用 `testing` feature 时提供领域实体与值对象的测试数据工厂，
//! 供 storage、services、app 等依赖 domain 的测试使用。
//!
//! # 功能启用
//!
//! 在依赖 domain 的 crate 的 `Cargo.toml` 中：
//!
//! ```toml
//! [dev-dependencies]
//! domain = { path = "../domain", features = ["testing"] }
//! ```
//!
//! 或在 `[dependencies]` 中需要测试工具时：
//!
//! ```toml
//! domain = { path = "../domain", features = ["testing"] }
//! ```
//!
//! # 实体工厂 (TestEntityFactory)
//!
//! 用于构建领域实体，带默认值，仅覆盖需要断言或参与行为的字段即可。
//!
//! | 方法 | 说明 |
//! |------|------|
//! | `branch_info()` | 分支信息 |
//! | `commit_info()` | 提交信息 |
//! | `pull_request_info()` | Pull Request 信息 |
//! | `stash_entry()` | Stash 条目 |
//! | `remote_info()` | 远程信息 |
//! | `repo_info()` | 仓库信息 |
//!
//! ```ignore
//! use domain::testing::TestEntityFactory;
//!
//! let branch = TestEntityFactory::branch_info()
//!     .with_name("feature/new-feature")
//!     .as_current()
//!     .with_upstream("origin/feature/new-feature")
//!     .build();
//!
//! let pr = TestEntityFactory::pull_request_info()
//!     .with_title("Add feature")
//!     .with_source_branch("feature/xyz")
//!     .with_target_branch("main")
//!     .build();
//! ```
//!
//! # 值对象工厂 (TestValueObjectFactory)
//!
//! 用于构建 PR 状态、PR 内容、文件状态、工作树状态等值对象；也提供常用预定义实例。
//!
//! | 方法 | 说明 |
//! |------|------|
//! | `pull_request_status()` | PR 状态构建器 |
//! | `pr_content()` | PR 标题/描述构建器 |
//! | `file_status_info()` | 文件状态信息构建器 |
//! | `working_tree_status()` | 工作树状态构建器 |
//! | `open_pr_status()` | 预定义 open 状态 |
//! | `merged_pr_status()` | 预定义 merged 状态 |
//! | `clean_working_tree()` | 预定义干净工作树 |
//!
//! ```ignore
//! use domain::testing::TestValueObjectFactory;
//!
//! let status = TestValueObjectFactory::pull_request_status()
//!     .merged(Some("2024-01-01T12:00:00Z"))
//!     .build();
//! let content = TestValueObjectFactory::pr_content()
//!     .with_title("My PR")
//!     .with_description("Description")
//!     .build();
//! let clean = TestValueObjectFactory::clean_working_tree();
//! assert!(clean.is_clean());
//! ```
//!
//! # 组合使用
//!
//! 实体构建器可接收值对象工厂产出的值：
//!
//! ```ignore
//! use domain::testing::{TestEntityFactory, TestValueObjectFactory};
//!
//! let pr = TestEntityFactory::pull_request_info()
//!     .with_title("Add feature")
//!     .with_status(TestValueObjectFactory::open_pr_status())
//!     .build();
//! ```
//!
//! # 运行 domain 自身测试
//!
//! ```bash
//! cargo test -p domain --features testing
//! ```

pub mod entity_factory;
pub mod value_object_factory;

pub use entity_factory::{
    BranchInfoBuilder, CommitInfoBuilder, PullRequestInfoBuilder, RemoteInfoBuilder,
    RepoInfoBuilder, StashEntryBuilder, TestEntityFactory,
};
pub use value_object_factory::{
    FileStatusInfoBuilder, PrContentBuilder, PullRequestStatusBuilder, TestValueObjectFactory,
    WorkingTreeStatusBuilder,
};
