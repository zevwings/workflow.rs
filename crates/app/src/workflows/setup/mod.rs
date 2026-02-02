//! 设置流程编排模块
//!
//! 编排完整的设置工作流，调用各个平台阶段。

pub mod orchestrator;

pub use orchestrator::run_setup_workflow;
