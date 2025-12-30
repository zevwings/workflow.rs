//! 交互式对话框模块
//!
//! 提供统一的交互式对话框接口，使用 `inquire` 和 `dialoguer` 作为后端实现。
//! 支持链式调用，提供更好的用户体验和代码可读性。
//!
//! **后端实现：**
//! - `InputDialog`, `SelectDialog`, `MultiSelectDialog`：使用 `inquire`
//! - `ConfirmDialog`：使用 `dialoguer`（支持单键自动完成和 Enter 使用默认值）
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
mod form;
mod input;
mod multi_select;
mod select;
#[doc(hidden)]
pub mod skip_config; // 内部 API，供测试代码使用（必须为 pub 以便测试代码访问）
mod types;

pub use confirm::ConfirmDialog;
pub use form::{
    Condition, ConditionEvaluator, ConditionOperator, ConditionValue, FieldDefaultValue,
    FormBuilder, FormField, FormFieldType, FormGroup, FormResult, FormStep, GroupConfig, StepType,
};
pub use input::InputDialog;
pub use multi_select::MultiSelectDialog;
pub use select::SelectDialog;

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Dialog Configuration Completeness Tests ====================

    /// 测试所有对话框的完整配置可以组合使用
    ///
    /// ## 测试目的
    /// 验证所有对话框类型（InputDialog, SelectDialog, MultiSelectDialog, ConfirmDialog）的完整配置方法可以链式调用，不会产生编译错误。
    ///
    /// ## 测试场景
    /// 1. 配置InputDialog的所有选项（default, validator, allow_empty）
    /// 2. 配置SelectDialog的所有选项（default）
    /// 3. 配置MultiSelectDialog的所有选项（default）
    /// 4. 配置ConfirmDialog的所有选项（default, cancel_message）
    ///
    /// ## 预期结果
    /// - 所有对话框的完整配置都能成功创建
    /// - 链式调用正常工作
    #[test]
    fn test_dialog_configuration_completeness_with_all_dialogs_configures_correctly() {
        // Arrange: 准备各种对话框的完整配置

        // Act & Assert: 验证所有对话框的完整配置都可以组合使用
        // InputDialog完整配置
        let _input = InputDialog::new("Enter value")
            .with_default("default")
            .with_validator(|s: &str| {
                if !s.is_empty() {
                    Ok(())
                } else {
                    Err("Empty".to_string())
                }
            })
            .allow_empty(false);

        // SelectDialog完整配置
        let _select = SelectDialog::new("Choose", vec!["A", "B", "C"]).with_default(0);

        // MultiSelectDialog完整配置
        let _multi = MultiSelectDialog::new("Choose", vec!["A", "B", "C"]).with_default(vec![0]);

        // ConfirmDialog完整配置
        let _confirm = ConfirmDialog::new("Continue?")
            .with_default(true)
            .with_cancel_message("Cancelled");
    }

    /// 测试不同对话框类型保持类型安全
    ///
    /// ## 测试目的
    /// 验证不同类型的对话框（InputDialog, SelectDialog, MultiSelectDialog, ConfirmDialog）在类型系统中保持类型安全，不能互相混淆。
    ///
    /// ## 测试场景
    /// 1. 创建不同类型的对话框
    /// 2. 验证类型正确（通过编译验证）
    ///
    /// ## 预期结果
    /// - 所有对话框类型正确
    /// - 类型系统能够区分不同对话框类型
    #[test]
    fn test_dialog_type_safety_with_different_types_maintains_type_safety() {
        // Arrange: 准备不同类型的对话框

        // Act: 创建不同类型的对话框
        let _input: InputDialog = InputDialog::new("Input");
        let _select: SelectDialog<&str> = SelectDialog::new("Select", vec!["A"]);
        let _multi: MultiSelectDialog<&str> = MultiSelectDialog::new("Multi", vec!["A"]);
        let _confirm: ConfirmDialog = ConfirmDialog::new("Confirm");

        // Assert: 验证类型正确（通过编译验证）
    }

    /// 测试对话框错误处理结构存在
    ///
    /// ## 测试目的
    /// 验证对话框具有错误处理结构（主要验证结构正确，实际错误处理在prompt()方法中，需要用户交互才能测试）。
    ///
    /// ## 测试场景
    /// 1. 创建对话框
    /// 2. 验证对话框可以创建，错误处理结构存在
    ///
    /// ## 注意事项
    /// - 实际错误处理在prompt()方法中
    /// - 需要用户交互才能测试实际错误处理
    ///
    /// ## 预期结果
    /// - 对话框创建成功
    /// - 错误处理结构存在
    #[test]
    fn test_dialog_error_handling_structure_with_dialog_has_error_handling() {
        // Arrange: 准备对话框
        // 注意：这个测试主要验证错误处理的结构正确
        // 实际错误处理在prompt()方法中，需要用户交互才能测试

        // Act: 创建对话框
        let _dialog = InputDialog::new("Test");

        // Assert: 验证对话框可以创建，错误处理结构存在
    }
}
