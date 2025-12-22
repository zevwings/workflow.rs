//! 交互式表单模块
//!
//! 提供表单构建和执行功能，支持链式调用、条件字段、验证器等特性。
//!
//! ## 特性
//!
//! - **链式构建**：使用 FormBuilder 链式构建表单
//! - **条件字段**：根据之前输入动态显示字段
//! - **统一结果收集**：FormResult 统一管理所有字段值
//! - **多种字段类型**：Text、Password、Selection、Confirmation
//! - **验证器和默认值**：支持自定义验证和默认值
//!
//! ## 使用示例
//!
//! ### 基本用法
//!
//! ```rust,no_run
//! use workflow::base::dialog::form::FormBuilder;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let result = FormBuilder::new()
//!     .add_text("name", "Enter your name")
//!         .required()
//!         .default("John Doe")
//!     .add_password("password", "Enter password")
//!         .required()
//!     .run()?;
//!
//! let name = result.get_required("name")?;
//! # Ok(())
//! # }
//! ```
//!
//! ### 条件字段
//!
//! ```rust,no_run
//! use workflow::base::dialog::form::FormBuilder;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let result = FormBuilder::new()
//!     .add_selection("provider", "Select provider", vec!["openai".to_string(), "custom".to_string()])
//!     .when("provider", "custom", |f| {
//!         f.add_text("url", "Enter URL").required()
//!     })
//!     .run()?;
//! # Ok(())
//! # }
//! ```
//!
//! ### 验证器
//!
//! ```rust,no_run
//! use workflow::base::dialog::form::FormBuilder;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let result = FormBuilder::new()
//!     .add_text("email", "Enter email")
//!         .required()
//!         .validate(|input: &str| {
//!             if input.contains('@') {
//!                 Ok(())
//!             } else {
//!                 Err("Invalid email format".to_string())
//!             }
//!         })
//!     .run()?;
//! # Ok(())
//! # }
//! ```

mod builder;
mod executor;
mod types;

pub use builder::FormBuilder;
pub use types::{Condition, ConditionOperator, ConditionValue, FormField, FormFieldType, FormResult};
