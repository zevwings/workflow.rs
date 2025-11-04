//! Settings 模块
//! 用于管理应用程序的各种设置和配置

#[allow(clippy::module_inception)]
pub mod settings;

// 导出公共类型和函数
pub use settings::Settings;
