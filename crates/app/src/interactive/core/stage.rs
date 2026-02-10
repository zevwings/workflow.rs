//! 工作流阶段抽象模块
//!
//! 定义 `WorkflowStage` trait 和 `WorkflowExecutor`，统一设置和验证流程。

use crate::bootstrap;
use crate::interactive::core::context::{WorkflowContext, WorkflowMode};
use crate::interactive::display::VerificationResultFormatter;
use domain::{GlobalConfig, VerificationService};
use prompt::{br, spinner_then, success, warning};
use std::error::Error;

/// 工作流中的一个阶段（如 Log、GitHub、LLM）
pub trait WorkflowStage {
    /// 阶段的名称（如 "Log"、"GitHub"）
    fn stage_name(&self) -> &'static str;

    /// 配置此阶段的设置
    ///
    /// 此方法应向用户展示表单并通过 `context` 更新 `settings` 对象。
    /// 它不应保存配置；保存由执行器或调用者处理。
    fn configure(&self, context: &mut WorkflowContext) -> Result<(), Box<dyn Error>>;

    /// 检查阶段是否已配置
    fn is_configured(&self, settings: &GlobalConfig) -> bool;

    /// 验证此阶段的配置
    ///
    /// 此方法应调用 `VerificationService` 上的相应方法。
    fn verify(
        &self,
        service: &dyn VerificationService,
    ) -> Result<Box<dyn VerificationResultFormatter>, Box<dyn Error>>;

    /// 是否需要显示 spinner
    ///
    /// 返回 `true` 表示在验证过程中显示 spinner，返回 `false` 则不显示。
    /// 默认返回 `true`。
    fn needs_spinner(&self) -> bool {
        true
    }
}

/// 工作流阶段的执行器
pub struct WorkflowExecutor<'a> {
    stage: &'a dyn WorkflowStage,
}

impl<'a> WorkflowExecutor<'a> {
    /// 为给定阶段创建新的执行器
    pub fn new(stage: &'a dyn WorkflowStage) -> Self {
        Self { stage }
    }

    /// 使用现有上下文运行阶段的设置过程
    /// 不保存配置
    pub fn run_setup(&self, context: &mut WorkflowContext) -> Result<(), Box<dyn Error>> {
        self.stage.configure(context)?;
        success!("{} configuration updated.", self.stage.stage_name());
        Ok(())
    }

    /// 作为独立命令运行设置过程
    /// 加载配置，运行设置，然后保存配置
    pub fn run_command_setup(&self) -> Result<(), Box<dyn Error>> {
        let mut context = WorkflowContext::load(WorkflowMode::Command)?;

        self.run_setup(&mut context)?;

        context.save()?;
        br!();

        Ok(())
    }

    /// 运行阶段的验证过程
    pub fn run_verify(&self) -> Result<(), Box<dyn Error>> {
        let context = WorkflowContext::load(WorkflowMode::Command)?;
        let settings = context.settings();
        let stage_name = self.stage.stage_name();

        if !self.stage.is_configured(settings) {
            warning!("{} is not configured. Skipping verification.", stage_name);
            return Ok(());
        }

        let service = bootstrap::get_verification_service();
        let label = format!("Verifying {} configuration...", stage_name);

        let result = if self.stage.needs_spinner() {
            spinner_then!(
                &label,
                || self.stage.verify(service.as_ref()),
                |res: &dyn VerificationResultFormatter| res.format(),
            )
        } else {
            let verify_result = self.stage.verify(service.as_ref());
            if let Ok(ref res) = verify_result {
                res.format();
            }
            verify_result
        };

        if let Err(err) = &result {
            warning!("{} configuration verification failed: {}", stage_name, err);
        }

        Ok(())
    }
}
