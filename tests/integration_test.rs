//! 集成测试入口文件
//!
//! 此文件用于组织 tests/ 目录下的所有模块化测试。
//! Rust 的测试系统会自动识别 tests/ 目录下的每个 .rs 文件作为独立的测试 crate。
//! 通过此入口文件，我们可以组织目录结构中的测试模块。

// 引入各个模块的测试
mod base;
mod branch; // 新增：Branch 模块测试
mod cli;
mod commit; // 新增：Commit 模块测试
mod completion;
mod git;
mod http;
mod integration;
mod jira;
mod pr;
mod proxy;
// mod rollback; // 暂时禁用：Rollback 模块测试 (rollback/ 目录下所有文件已禁用)
mod template; // 新增：Template 模块测试

// 共享测试工具
mod common;
