//! Resilience 模块集成测试
//!
//! 包含 resilience 模块的集成测试，主要测试：
//! - 并发场景
//! - 系统级行为（资源限制、线程泄漏）
//! - 总超时和指数退避机制

pub mod retry;
pub mod timeout;
