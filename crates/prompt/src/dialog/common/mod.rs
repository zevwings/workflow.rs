//! 共享基础设施模块
//!
//! 提供所有对话框类型共享的基础功能：
//! - 原始模式管理
//! - 取消提示渲染

mod cancel;
mod raw_mode;

pub use cancel::print_cancelled_message;
pub use raw_mode::RawModeGuard;
