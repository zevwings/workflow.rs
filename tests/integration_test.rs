//! 集成测试入口文件
//!
//! 此文件用于组织 tests/ 目录下的所有模块化测试。
//! Rust 的测试系统会自动识别 tests/ 目录下的每个 .rs 文件作为独立的测试 crate。
//! 通过此入口文件，我们可以组织目录结构中的测试模块。

// 引入各个模块的测试
mod alias;
mod branch;
mod cli;
mod commit;
mod completion;
mod concurrent;
mod dialog;
mod git;
mod http;
mod indicator;
mod integration;
mod jira;
mod llm;
mod logger;
mod mcp;
mod pr;
mod prompt;
mod proxy;
mod rollback;
mod settings;
mod shell;
mod template;
mod util;

// 共享测试工具
mod common;
mod utils;
