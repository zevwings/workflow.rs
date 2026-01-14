//! 共享基础设施模块
//!
//! 提供所有对话框类型共享的基础功能：
//! - 原始模式管理

mod raw_mode;

pub use raw_mode::RawModeGuard;
