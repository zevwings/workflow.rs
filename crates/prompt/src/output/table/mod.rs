//! 表格渲染模块

mod builder;
mod render;
mod row;
mod tabled;
mod width;

pub use builder::{Alignment, TableBuilder, TableStyle};
pub use tabled::Tabled;

/// 便捷函数
pub fn table(headers: Vec<impl Into<String>>) -> TableBuilder {
    TableBuilder::new(headers)
}
