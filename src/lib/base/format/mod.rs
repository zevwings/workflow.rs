//! 格式化工具模块
//!
//! 提供各种格式化功能，包括文件大小格式化、错误消息格式化、路径格式化、日期时间格式化和敏感字符串处理等。
//!
//! ## 模块结构
//!
//! - `message` - 消息格式化器（错误消息、操作消息、进度信息）
//! - `display` - 显示格式化器（路径、列表项、键值对、文件大小）
//! - `date` - 日期时间格式化器（文档时间戳、Unix 时间戳）
//! - `sensitive` - 敏感字符串处理（敏感值隐藏）

pub mod date;
pub mod display;
pub mod message;
pub mod sensitive;

// 重新导出子模块的结构体和 trait
pub use date::{
    format_document_timestamp, format_last_updated, format_last_updated_with_time, DateFormat,
    Timezone,
};
pub use display::DisplayFormatter;
pub use message::MessageFormatter;
pub use sensitive::Sensitive;
