//! Workflow 库入口
//!
//! 这个模块重新导出了 Workflow CLI 的所有公共 API，方便其他模块使用。
//! 采用三层架构设计：
//! - **CLI 入口层** (`bin/`, `main.rs`): 命令行参数解析和命令分发
//! - **命令封装层** (`commands/`): CLI 命令封装，处理用户交互
//! - **核心业务逻辑层** (`lib/`): 所有业务逻辑实现

// 核心库模块声明
#[path = "lib/git/mod.rs"]
pub mod git;
#[path = "lib/http/mod.rs"]
pub mod http;
#[path = "lib/jira/mod.rs"]
pub mod jira;
#[path = "lib/llm/mod.rs"]
pub mod llm;
#[path = "lib/log/mod.rs"]
pub mod log;
#[path = "lib/pr/mod.rs"]
pub mod pr;
#[path = "lib/settings/mod.rs"]
pub mod settings;
#[path = "lib/utils/mod.rs"]
pub mod utils;

// 命令模块声明
#[path = "commands/mod.rs"]
pub mod commands;

// 重新导出所有公共 API，方便外部使用
pub use git::*;
pub use http::{Authorization, HttpClient, HttpResponse};
pub use jira::*;
pub use llm::*;
pub use log::*;
pub use pr::{
    extract_pull_request_id_from_url, generate_branch_name, generate_commit_title,
    generate_pull_request_body, Codeup, GitHub, PlatformProvider, TYPES_OF_CHANGES,
};
pub use settings::*;
pub use utils::*;
