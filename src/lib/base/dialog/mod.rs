//! 交互式对话框模块
//!
//! 提供统一的交互式对话框接口，使用 `inquire` 作为后端实现。
//! 支持链式调用，提供更好的用户体验和代码可读性。
//!
//! ## 对话框类型
//!
//! - `InputDialog` - 文本输入对话框
//! - `SelectDialog` - 单选对话框
//! - `MultiSelectDialog` - 多选对话框
//! - `ConfirmDialog` - 确认对话框
//!
//! ## 使用示例
//!
//! ### InputDialog
//!
//! ```rust,no_run
//! use workflow::base::dialog::InputDialog;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // 简单输入
//! let name = InputDialog::new("Enter your name")
//!     .prompt()?;
//!
//! // 带默认值
//! let email = InputDialog::new("Enter email")
//!     .with_default("user@example.com")
//!     .prompt()?;
//!
//! // 带验证器
//! let age = InputDialog::new("Enter age")
//!     .with_validator(|input: &str| {
//!         if input.parse::<u32>().is_ok() {
//!             Ok(())
//!         } else {
//!             Err("Please enter a valid number".to_string())
//!         }
//!     })
//!     .prompt()?;
//!
//! // 允许空值
//! let optional = InputDialog::new("Enter value (optional)")
//!     .allow_empty(true)
//!     .prompt()?;
//! # Ok(())
//! # }
//! ```
//!
//! ### SelectDialog
//!
//! ```rust,no_run
//! use workflow::base::dialog::SelectDialog;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // 基本用法
//! let options = vec!["Option 1", "Option 2", "Option 3"];
//! let selected = SelectDialog::new("Choose an option", options)
//!     .with_default(0)
//!     .prompt()?;
//! // selected 是 "Option 1" 或 "Option 2" 或 "Option 3"
//!
//! // 使用模糊匹配过滤器（适用于选项较多时）
//! let branches = vec!["feature/user-auth", "feature/payment", "bugfix/login"];
//! let selected = SelectDialog::new("选择分支", branches)
//!     // 模糊匹配默认启用，支持输入关键词过滤
//!     .prompt()?;
//! # Ok(())
//! # }
//! ```
//!
//! ### MultiSelectDialog
//!
//! ```rust,no_run
//! use workflow::base::dialog::MultiSelectDialog;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let options = vec!["Option 1", "Option 2", "Option 3"];
//! let selected = MultiSelectDialog::new("Choose options", options)
//!     .prompt()?;
//! // selected 是 Vec<&str>，包含选中的选项
//! # Ok(())
//! # }
//! ```
//!
//! ### ConfirmDialog
//!
//! ```rust,no_run
//! use workflow::base::dialog::ConfirmDialog;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // 简单确认
//! let confirmed = ConfirmDialog::new("Continue?")
//!     .with_default(true)
//!     .prompt()?;
//!
//! // 取消时返回错误
//! ConfirmDialog::new("This operation cannot be undone. Continue?")
//!     .with_default(false)
//!     .with_cancel_message("Operation cancelled.")
//!     .prompt()?;
//! # Ok(())
//! # }
//! ```

mod confirm;
mod input;
mod multi_select;
mod select;
mod types;

pub use confirm::ConfirmDialog;
pub use input::InputDialog;
pub use multi_select::MultiSelectDialog;
pub use select::SelectDialog;
