//! HTTP 测试辅助工具
//!
//! 提供 Mock 服务器管理和测试数据工厂，用于简化 HTTP 相关测试。
//!
//! # 功能启用
//!
//! ## 在本 crate 中使用
//!
//! 在本 crate 的测试中，testing 模块会自动可用（通过 `#[cfg(test)]`）：
//!
//! ```ignore
//! #[cfg(test)]
//! mod tests {
//!     use crate::testing::{TestDataFactory, MockServerManager};
//!
//!     #[test]
//!     fn test_something() {
//!         let pr = TestDataFactory::github_pr().build();
//!         // ...
//!     }
//! }
//! ```
//!
//! ## 在其他 crate 中使用
//!
//! 在其他 crate 的 `Cargo.toml` 中启用 testing feature：
//!
//! ```toml
//! [dev-dependencies]
//! http = { workspace = true, features = ["testing"] }
//! ```
//!
//! 然后在测试中使用：
//!
//! ```ignore
//! #[cfg(test)]
//! mod tests {
//!     use http::testing::{TestDataFactory, MockServerManager};
//!
//!     #[test]
//!     fn test_api_integration() {
//!         let mut manager = MockServerManager::new();
//!         let pr = TestDataFactory::github_pr()
//!             .with_title("My Feature")
//!             .build();
//!
//!         let _mock = manager.setup_github_pr_list(vec![pr]);
//!         let github_url = manager.url("github").unwrap();
//!         // 使用 github_url 进行测试...
//!     }
//! }
//! ```
//!
//! # 使用示例
//!
//! ## 测试数据工厂
//!
//! 使用构建器模式创建测试数据：
//!
//! ```ignore
//! use http::testing::TestDataFactory;
//!
//! // GitHub PR 数据
//! let pr = TestDataFactory::github_pr()
//!     .with_title("Add new feature")
//!     .with_number(123)
//!     .build();
//!
//! // Jira Issue 数据
//! let issue = TestDataFactory::jira_issue()
//!     .with_key("PROJ-123")
//!     .with_summary("Fix bug")
//!     .build();
//! ```
//!
//! ## Mock 服务器
//!
//! 两种使用方式：
//!
//! ### MockServer - 简单场景
//!
//! ```ignore
//! use http::testing::MockServer;
//!
//! let mut server = MockServer::new();
//! let _mock = server.mock("GET", "/test")
//!     .with_status(200)
//!     .with_body("response")
//!     .create();
//! let url = server.url();
//! ```
//!
//! ### MockServerManager - 复杂场景
//!
//! ```ignore
//! use http::testing::{MockServerManager, TestDataFactory};
//!
//! let mut manager = MockServerManager::new();
//! let pr = TestDataFactory::github_pr().build();
//! let _mock = manager.setup_github_pr_list(vec![pr]);
//! let url = manager.url("github").unwrap();
//! ```
//!
//! # 最佳实践
//!
//! 1. **使用测试数据工厂而不是硬编码** - 使用 `TestDataFactory` 创建测试数据
//! 2. **Mock 对象生命周期** - Mock 对象需要保持存活直到测试结束
//! 3. **明确的测试场景命名** - 使用描述性的测试函数名
//!
//! # 注意事项
//!
//! - **Feature Flag**: 在其他 crate 使用时，必须显式启用 `testing` feature
//! - **生产代码隔离**: testing 模块不会被编译到生产代码中
//! - **端口冲突**: 每个测试会自动分配不同的端口，不会冲突

mod data_factory;
mod mock_server;

pub use data_factory::{ConfigBuilder, GitHubPRBuilder, JiraIssueBuilder, TestDataFactory};
pub use mock_server::{MockServer, MockServerManager};
