//! 配置初始化命令
//!
//! - 读取现有 `workflow.toml`（如果不存在则从默认值开始）
//! - 交互式配置 GitHub / Jira / LLM / 日志
//! - 保存配置并给出提示

use crate::interactive::setup::run_setup_workflow;

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
        run_setup_workflow()
    }
}
