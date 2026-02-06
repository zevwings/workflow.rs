//! 工作流上下文模块
//!
//! 管理工作流执行上下文，包括配置加载和保存。

use std::error::Error;

use domain::{GlobalConfig, ServiceError};
use prompt::{br, success};

use crate::registry::{get_global_config_repository, get_path_service};

/// 工作流执行模式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkflowMode {
    /// 作为初始设置向导的一部分运行
    Setup,
    /// 作为独立命令运行（如 `workflow log setup`）
    Command,
}

/// 工作流执行上下文
pub struct WorkflowContext {
    settings: GlobalConfig,
    mode: WorkflowMode,
}

impl WorkflowContext {
    /// 从默认路径加载配置，使用指定的模式
    pub fn load(mode: WorkflowMode) -> Result<Self, Box<dyn Error>> {
        let config_service = get_global_config_repository();
        let settings =
            config_service.load().map_err(|e: ServiceError| Box::new(e) as Box<dyn Error>)?;

        Ok(Self { settings, mode })
    }

    /// 使用默认设置创建新上下文（用于初始设置）
    pub fn new_with_defaults(mode: WorkflowMode) -> Self {
        Self {
            settings: GlobalConfig::default(),
            mode,
        }
    }

    /// 获取当前执行模式
    pub fn mode(&self) -> WorkflowMode {
        self.mode
    }

    /// 获取设置的引用
    pub fn settings(&self) -> &GlobalConfig {
        &self.settings
    }

    /// 获取设置的可变引用
    pub fn settings_mut(&mut self) -> &mut GlobalConfig {
        &mut self.settings
    }

    /// 保存配置到默认路径
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let config_service = get_global_config_repository();
        config_service
            .save(&self.settings)
            .map_err(|e| format!("Failed to save configuration: {}", e))?;

        let path_service = get_path_service();
        let workflow_config_filepath = path_service.get_workflow_config_filepath()?;

        br!();
        success!(
            "Configuration saved to: {}",
            workflow_config_filepath.display()
        );

        Ok(())
    }
}
