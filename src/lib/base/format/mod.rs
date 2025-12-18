//! 格式化工具模块
//!
//! 提供各种格式化功能，包括文件大小格式化、错误消息格式化、路径格式化等。
//!
//! ## 模块结构
//!
//! - `message` - 消息格式化器（错误消息、操作消息、进度信息）
//! - `display` - 显示格式化器（路径、列表项、键值对、文件大小）

pub mod message;
pub mod display;

// 重新导出子模块的结构体
pub use message::MessageFormatter;
pub use display::DisplayFormatter;
