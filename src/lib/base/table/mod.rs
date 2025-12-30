//! 表格输出工具
//!
//! 提供统一的表格输出接口，使用 tabled 库。
//!
//! ## 功能特性
//!
//! - 自动格式化表格
//! - 支持自定义样式和边框
//! - 支持列对齐和宽度控制
//! - 支持紧凑模式和完整模式
//! - 支持链式配置
//!
//! ## 使用示例
//!
//! ```rust
//! use tabled::Tabled;
//! use workflow::base::table::{TableBuilder, TableStyle};
//!
//! #[derive(Tabled, Clone)]
//! struct User {
//!     name: String,
//!     age: u32,
//! }
//!
//! let users = vec![
//!     User { name: "Alice".to_string(), age: 30 },
//!     User { name: "Bob".to_string(), age: 25 },
//! ];
//!
//! // 链式调用方式
//! let output = TableBuilder::new(users.clone())
//!     .with_title("Users")
//!     .with_style(TableStyle::Modern)
//!     .render();
//! println!("{}", output);
//!
//! // 或者使用 Display trait
//! println!("{}", TableBuilder::new(users));
//! ```

#[allow(clippy::module_inception)]
mod table;

pub use table::{TableBuilder, TableStyle};
