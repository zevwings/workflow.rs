//! 配置管理模块
//!
//! 提供统一的配置管理，支持三层配置优先级：
//! 1. 默认配置（defaultConfig）：系统默认值
//! 2. 全局配置（globalConfig）：用户设置的全局配置
//! 3. 局部配置（localConfig）：每次调用时的局部配置
//!
//! 优先级：defaultConfig < globalConfig < localConfig

#[allow(clippy::module_inception)]
mod config;
mod manager;

pub use config::{with_result_title, PromptConfig};
pub use manager::ConfigManager;
