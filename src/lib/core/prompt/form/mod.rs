//! 表单模块
//!
//! 提供表单构建器，支持组合多个字段进行输入
//!
//! 支持两种模式：
//! - 简单模式：直接添加字段，使用字段条件控制显示
//! - Group/Step 模式：使用组和步骤组织字段，支持可选组和步骤条件

mod builder;
mod executor;
mod field;
mod types;

pub use builder::FormBuilder;
pub use executor::FormExecutor;
pub use field::{
    Condition, ConfirmFormField, InputFormField, MultiSelectFormField, NestedFormField,
    PasswordFormField, SelectFormField,
};
pub use types::{FormResult, GroupConfig};

/// 便捷函数：创建表单构建器
pub fn form() -> FormBuilder {
    FormBuilder::new()
}
