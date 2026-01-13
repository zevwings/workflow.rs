//! 配置管理器
//!
//! 提供统一的配置管理接口，支持默认配置、全局配置和局部配置的层次结构

use super::config::PromptConfig;

/// 配置管理器
/// 提供统一的配置管理接口，支持默认配置、全局配置和局部配置的层次结构
pub struct ConfigManager {
    /// 默认配置（系统默认值）
    default_config: PromptConfig,
    /// 全局配置（用户设置的全局配置）
    global_config: PromptConfig,
}

impl ConfigManager {
    /// 创建配置管理器
    pub fn new(default_config: PromptConfig) -> Self {
        Self {
            default_config,
            global_config: PromptConfig::new(),
        }
    }

    /// 设置全局配置
    /// 全局配置会与默认配置合并，非 None 字段会覆盖默认配置
    pub fn set_global_config(&mut self, config: PromptConfig) {
        self.global_config = config;
    }

    /// 获取全局配置
    pub fn get_global_config(&self) -> &PromptConfig {
        &self.global_config
    }

    /// 构建最终配置
    /// 按照优先级合并：defaultConfig < globalConfig < localConfig
    /// 返回合并后的配置
    pub fn build_config(&self, local_config: Option<&PromptConfig>) -> PromptConfig {
        // 首先合并默认配置和全局配置
        let merged = PromptConfig::fill_defaults(&self.global_config, &self.default_config);

        // 如果有局部配置，继续合并
        if let Some(local) = local_config {
            PromptConfig::merge(&merged, local)
        } else {
            merged
        }
    }

    /// 重置全局配置为空配置
    pub fn reset_global_config(&mut self) {
        self.global_config = PromptConfig::new();
    }

    /// 获取默认配置（只读）
    pub fn get_default_config(&self) -> &PromptConfig {
        &self.default_config
    }
}
