//! 配置适配器模块
//!
//! 提供各种配置源的适配器实现。

pub mod settings;

// 重新导出适配器
pub use settings::SettingsAdapter;
