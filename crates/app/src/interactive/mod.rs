//! 交互式工作流模块
//!
//! 提供 CLI 命令的用户交互逻辑，包括配置向导、验证流程、显示格式化等。

pub mod core;
pub mod display;
pub mod platforms;
pub mod setup;

// 重新导出常用接口
pub use core::{WorkflowContext, WorkflowExecutor, WorkflowMode, WorkflowStage};
pub use platforms::{github_stage, jira_stage, llm_stage, log_stage};
