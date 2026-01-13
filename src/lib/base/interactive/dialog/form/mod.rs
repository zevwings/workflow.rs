//! 表单模块
//!
//! 提供表单构建器，支持组合多个字段进行输入

mod builder;
mod executor;
mod field;
mod result;

pub use builder::FormBuilder;
pub use executor::FormExecutor;
pub use field::{
    Condition, ConfirmFormField, InputFormField, MultiSelectFormField, NestedFormField,
    PasswordFormField, SelectFormField,
};
pub use result::FormResult;

/// 便捷函数：创建表单构建器
pub fn form() -> FormBuilder {
    FormBuilder::new()
}
