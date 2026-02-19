//! 命令实现模块
//!
//! CLI 命令封装，负责参数解析、用户交互、输出格式化

// 共享参数模块
pub mod args;

// CLI 定义模块（顶层命令结构）
pub mod cli;

// 命令模块
pub mod alias;
pub mod branch;
pub mod check;
pub mod commit;
pub mod completion;
pub mod github;
pub mod install;
pub mod jira;
pub mod llm;
pub mod log;
pub mod pr;
pub mod repo;
#[cfg(feature = "develop")]
pub mod rollback;
pub mod setup;
pub mod stash;
#[cfg(feature = "develop")]
pub mod sync;
pub mod tag;
pub mod uninstall;
pub mod update;
pub mod version;
