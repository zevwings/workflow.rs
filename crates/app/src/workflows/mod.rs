//! 交互式工作流模块
//!
//! 提供 CLI 命令的用户交互逻辑，包括配置向导、验证流程、显示格式化等。

pub mod core;
pub mod display;
pub mod platforms;
pub mod setup;
pub mod utils;

// 重新导出常用接口
pub use core::{WorkflowContext, WorkflowExecutor, WorkflowMode, WorkflowStage};
pub use setup::run_setup_workflow;
pub use utils::{
    generate_branch_name_from_template, get_jira_id_interactive, get_jira_id_interactive_optional,
    to_slug,
};
