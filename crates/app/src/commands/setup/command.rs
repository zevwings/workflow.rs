//! 配置初始化命令
//!
//! - 读取现有 `workflow.toml`（如果不存在则从默认值开始）
//! - 交互式配置 GitHub / Jira / LLM / 日志
//! - 保存配置并给出提示

use prompt::{br, info, is_user_cancelled, separator, success, warning};

use crate::interactive::{
    github_stage, jira_stage, llm_stage, log_stage, WorkflowContext, WorkflowExecutor,
    WorkflowMode, WorkflowStage,
};

/// Setup 命令
pub struct SetupCommand;

impl Default for SetupCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl SetupCommand {
    /// 创建新的 SetupCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow setup` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        separator!('=', 80, "Workflow Setup");
        br!();
        info!("Starting Workflow CLI initialization...");

        let mut context = WorkflowContext::load(WorkflowMode::Setup)?;

        let stages: Vec<&dyn WorkflowStage> =
            vec![jira_stage(), github_stage(), llm_stage(), log_stage()];

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
}
