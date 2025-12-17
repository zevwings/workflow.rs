//! CLI 模块测试
//!
//! 包含 CLI 命令层的所有测试文件。

// pub mod basic_cli; // 暂时禁用：基础 CLI 测试 (basic_cli.rs.disabled)
pub mod branch;
pub mod check;
pub mod commit;
pub mod config;
pub mod github;
// pub mod integration_cli; // 暂时禁用：CLI 集成测试 (integration_cli.rs.disabled)
pub mod jira;
pub mod lifecycle;
pub mod llm;
pub mod log;
pub mod migrate;
pub mod pr;
// pub mod proxy; // 暂时禁用：代理相关CLI测试 (proxy.rs.disabled)
pub mod repo;
pub mod stash;
