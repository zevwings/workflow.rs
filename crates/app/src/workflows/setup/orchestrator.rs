//! 工作流设置编排模块
//!
//! 使用阶段架构编排完整的设置工作流。

use crate::workflows::core::context::{WorkflowContext, WorkflowMode};
use crate::workflows::core::stage::{WorkflowExecutor, WorkflowStage};
use crate::workflows::platforms::{
    github::github_stage, jira::jira_stage, llm::llm_stage, log::log_stage,
};
use prompt::{br, info, is_user_cancelled, separator, success, warning};
use std::error::Error;

/// 运行完整的设置工作流
pub fn run_setup_workflow() -> Result<(), Box<dyn Error>> {
    separator!('=', 80, "Workflow Setup");
    br!();
    info!("Starting Workflow CLI initialization...");

    let mut context = WorkflowContext::load(WorkflowMode::Setup)?;

    let stages: Vec<&dyn WorkflowStage> = vec![
        jira_stage(),
        github_stage(),
        llm_stage(),
        log_stage(),
    ];

    for stage in &stages {
        let executor = WorkflowExecutor::new(*stage);
        if let Err(err) = executor.run_setup(&mut context) {
            let err_msg = err.to_string();
            if is_user_cancelled(&err_msg) {
                br!();
                info!("Setup cancelled by user.");
                return Ok(());
            }
            warning!(
                "{} configuration failed or skipped: {}",
                stage.stage_name(),
                err
            );
        }
    }

    context.save()?;

    br!();
    separator!('─', 80, "Verification");
    br!();

    for stage in &stages {
        let executor = WorkflowExecutor::new(*stage);
        if let Err(err) = executor.run_verify() {
            warning!("{} verification failed: {}", stage.stage_name(), err);
        }
        br!();
    }

    br!();
    success!("Initialization completed successfully!");
    info!("You can now use the Workflow CLI commands.");

    Ok(())
}
