//! 交互式对话框模块
//!
//! 本模块提供所有交互式对话框功能，需要用户输入或选择：
//! - 输入对话框（input）：文本输入，支持密码模式
//! - 确认对话框（confirm）：Yes/No 选择
//! - 单选对话框（select）：从选项列表中选择一个
//! - 多选对话框（multiselect）：从选项列表中选择多个

mod common;
mod confirm;
mod input;
mod selection;

/// Result 类型别名（使用 crate::error::Result）
pub use crate::error::Result;

// ============================================================================
// 共享常量定义
// ============================================================================

/// 提示符前缀（用于所有对话框类型的提示行）
pub const PROMPT_PREFIX: &str = "? ";

/// 结果前缀（用于显示用户选择/输入的结果）
pub const RESULT_PREFIX: &str = "> ";

/// 密码显示掩码（用于密码输入时隐藏实际内容）
pub const PASSWORD_MASK: &str = "****";

/// 选中项前缀（用于 select/multiselect 中当前光标位置的选项）
pub const SELECTED_PREFIX: &str = "> ";

/// 未选中项前缀（用于 select/multiselect 中非当前光标位置的选项）
pub const UNSELECTED_PREFIX: &str = "  ";

// 重新导出公共 API
pub use common::RawModeGuard;
pub use confirm::ConfirmBuilder;
pub use input::{validators, InputBuilder, ValidationResult, Validator};
pub use selection::{FuzzyFilter, MultiSelectBuilder, SelectBuilder};
