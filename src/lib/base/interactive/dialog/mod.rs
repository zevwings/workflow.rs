//! 交互式对话框模块
//!
//! 本模块提供所有交互式对话框功能，需要用户输入或选择：
//! - 输入对话框（input）：文本输入，支持密码模式
//! - 确认对话框（confirm）：Yes/No 选择
//! - 单选对话框（select）：从选项列表中选择一个
//! - 多选对话框（multiselect）：从选项列表中选择多个
//!
//! 注意：表单对话框（form）已迁移到 `base::interactive::form`

mod confirm;
mod error; // 错误类型定义
mod filter; // 模糊匹配过滤器
mod input;
mod multiselect;
mod raw_mode; // 原始模式管理
mod renderer; // 内部使用，不导出
mod select;

pub use confirm::ConfirmBuilder;
pub use error::{PromptError, Result};
pub use filter::FuzzyFilter;
pub use input::{validators, InputBuilder, Validator};
pub use multiselect::MultiSelectBuilder;
pub use select::SelectBuilder;
