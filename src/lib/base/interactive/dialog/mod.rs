//! 交互式对话框模块
//!
//! 本模块提供所有交互式对话框功能，需要用户输入或选择：
//! - 输入对话框（input）：文本输入，支持密码模式
//! - 确认对话框（confirm）：Yes/No 选择
//! - 单选对话框（select）：从选项列表中选择一个
//! - 多选对话框（multiselect）：从选项列表中选择多个
//! - 表单对话框（form）：组合多个字段的复杂表单

mod confirm;
mod form;
mod input;
mod multiselect;
mod select;

pub use confirm::{confirm, ConfirmBuilder};
pub use form::{
    form, Condition, ConfirmFormField, FormBuilder, FormExecutor, FormResult, InputFormField,
    MultiSelectFormField, NestedFormField, PasswordFormField, SelectFormField,
};
pub use input::{input, validators, InputBuilder, Validator};
pub use multiselect::{multiselect, MultiSelectBuilder};
pub use select::{select, SelectBuilder};
