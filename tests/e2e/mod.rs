//! 端到端集成测试
//!
//! 包含完整的用户工作流测试。
//! 这些测试通常运行较慢，需要 Mock 服务器、Git 仓库等完整环境。

pub mod end_to_end;
pub mod jira;
pub mod performance;
pub mod scenarios;
pub mod workflow;
