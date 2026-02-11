//! Storage 测试辅助工具
//!
//! 在启用 `testing` feature 时提供 Git 测试辅助、性能监控以及 GitHub/Jira API fixtures，
//! 供本 crate 及依赖 storage 的测试、基准和示例使用。
//!
//! # 启用方式
//!
//! 在依赖本 crate 的 `Cargo.toml` 中：
//!
//! ```toml
//! [dev-dependencies]
//! storage = { path = "../storage", features = ["testing"] }
//! ```
//!
//! 若在 `[[bench]]` 或 `[[example]]` 中使用，需设置 `required-features = ["testing"]`。
//!
//! # 主要能力
//!
//! - **Git 测试**：[`setup_repo`]、[`setup_repo_with_config`] 等、[`noop_hook_service`]、[`with_isolated_git_env`]、
//!   [`TestRepoConfig`]、[`NoopHookService`]
//! - **性能监控**：[`performance`] 子模块（如 `PerformanceTimer`、`measure`、`PerformanceCollector`）
//! - **Fixtures**：[`GitHubFixtures`]、[`JiraFixtures`]
//!
//! # 示例
//!
//! ```rust,ignore
//! use storage::testing::{setup_repo, noop_hook_service, GitHubFixtures};
//!
//! #[test]
//! fn test_save_branch() {
//!     let (_tmp, ctx) = setup_repo();
//!     let hook = noop_hook_service();
//!     // 使用 ctx、hook 进行测试...
//! }
//!
//! let pr = GitHubFixtures::sample_pull_request();
//! ```
//!
//! # 运行
//!
//! - 测试：`cargo test -p storage --features testing`
//! - 基准：`cargo bench -p storage --features testing`
//! - 示例：`cargo run -p storage --example performance_monitoring --features testing`

pub mod git;
pub mod github_fixtures;
pub mod jira_fixtures;

// 重新导出 Git 测试常用函数与类型
pub use git::{
    noop_hook_service, setup_repo, setup_repo_with_branches, setup_repo_with_changes,
    setup_repo_with_commits, setup_repo_with_config, setup_repo_with_file, setup_repo_with_files,
    setup_repo_with_large_file, with_isolated_git_env, NoopHookService, TestRepoConfig,
};

// 重新导出性能监控工具（与原 git::testing::performance 等价）
pub use git::performance;

pub use github_fixtures::GitHubFixtures;
pub use jira_fixtures::JiraFixtures;
