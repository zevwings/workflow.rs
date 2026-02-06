//! 工具函数模块
//!
//! 提供通用的工具函数，包括分支处理、Jira 交互和验证辅助。

pub mod branch;
pub mod jira;

pub use branch::{generate_branch_name_from_template, to_slug};
pub use jira::{
    ensure_jira_status_config, extract_pr_id_from_url, get_jira_id_interactive,
    get_jira_id_interactive_optional,
};
