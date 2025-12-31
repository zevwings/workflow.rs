#![allow(
    clippy::overly_complex_bool_expr,
    clippy::module_inception,
    clippy::collapsible_match,
    clippy::to_string_trait_impl,
    clippy::unnecessary_unwrap,
    clippy::bool_comparison
)]

//! 模块级集成测试入口
//!
//! 包含所有模块的 public API 测试。
//! 这些测试通常运行较快，适合频繁运行。
//!
//! ## 测试类型
//!
//! - **模块级集成测试**：测试单个模块的 public API
//! - **运行速度**：快速（秒级）
//! - **依赖**：最小依赖，可以使用 Mock 服务器
//!
//! ## 运行方式
//!
//! ```bash
//! # 运行所有模块级集成测试
//! cargo test --test module_test
//!
//! # 运行特定模块的测试
//! cargo test --test module_test base::format
//! ```

// 引入各个模块的测试
mod base; // Base 模块测试（包含所有 base/* 子模块）
mod branch;
mod cli;
mod commands; // Commands 模块测试
mod commit;
mod completion;
mod git; // Git 模块测试
mod jira;
mod pr;
mod proxy;
mod repo;
mod rollback;
mod template;

// 共享测试工具
mod common;
