//! LLM 测试辅助工具
//!
//! 提供 Mock LLM 客户端和预定义响应 Fixtures，用于测试时避免真实 API 调用。
//!
//! ## 使用示例
//!
//! ```ignore
//! use llm::testing::{MockLLMClient, LLMFixtures};
//! use llm::{LLMClient, LLMRequestParameters};
//!
//! #[test]
//! fn test_commit_message_generation() {
//!     let client = MockLLMClient::new();
//!     client.add_response(LLMFixtures::commit_message());
//!
//!     let params = LLMRequestParameters::default();
//!     let content = client.call(&params).unwrap();
//!
//!     assert!(content.starts_with("feat:"));
//!     assert_eq!(client.call_count(), 1);
//! }
//! ```

pub mod fixtures;

pub use fixtures::LLMFixtures;
