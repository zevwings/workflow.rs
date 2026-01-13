//! 输出/显示功能模块
//!
//! 本模块提供所有输出和显示相关的功能，包括：
//! - 消息输出（message）：永久输出消息
//! - 表格渲染（table）：永久输出表格
//! - 加载指示器（spinner）：临时输出加载状态

mod message;
mod spinner;
mod table;

pub use message::{Message, MessageRef};
pub use spinner::{spinner, SpinnerBuilder};
pub use table::{table, Alignment, TableBuilder};
