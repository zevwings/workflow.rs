//! 分支管理命令

pub mod clean;
mod cli;
pub mod create;
pub mod ignore;
#[cfg(feature = "develop")]
pub mod infer_source;
pub mod remove;
pub mod rename;
pub mod switch;
pub(crate) mod utils;

// 重新导出 CLI 定义
pub use cli::{BranchSubcommand, IgnoreSubcommand};

// 重新导出工具函数（供跨模块使用）
pub use utils::{
    branch_type_from_branch_name, generate_branch_name_from_jira,
    generate_branch_name_from_template, select_branch_type, to_slug,
};
