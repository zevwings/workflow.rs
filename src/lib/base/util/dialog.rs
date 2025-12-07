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
//! use workflow::base::util::dialog::InputDialog;
//!
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
//! ```
//!
//! ### SelectDialog
//!
//! ```rust,no_run
//! use workflow::base::util::dialog::SelectDialog;
//!
//! let options = vec!["Option 1", "Option 2", "Option 3"];
//! let selected = SelectDialog::new("Choose an option", options)
//!     .with_default(0)
//!     .prompt()?;
//! // selected 是 "Option 1" 或 "Option 2" 或 "Option 3"
//! ```
//!
//! ### MultiSelectDialog
//!
//! ```rust,no_run
//! use workflow::base::util::dialog::MultiSelectDialog;
//!
//! let options = vec!["Option 1", "Option 2", "Option 3"];
//! let selected = MultiSelectDialog::new("Choose options", options)
//!     .prompt()?;
//! // selected 是 Vec<&str>，包含选中的选项
//! ```
//!
//! ### ConfirmDialog
//!
//! ```rust,no_run
//! use workflow::base::util::dialog::ConfirmDialog;
//!
//! // 简单确认
//! let confirmed = ConfirmDialog::new("Continue?")
//!     .with_default(true)
//!     .prompt()?;
//!
//! // 取消时返回错误
//! ConfirmDialog::new("This operation cannot be undone. Continue?")
//!     .with_default(false)
//     .with_cancel_message("Operation cancelled.")
//     .prompt()?;
//! ```

use anyhow::Result;
use inquire::{error::InquireError, validator::Validation, Confirm, MultiSelect, Select, Text};
use std::error::Error;
use std::sync::Arc;

/// Type alias for validator functions to reduce type complexity
type ValidatorFn = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// 文本输入对话框
///
/// 提供文本输入功能，支持默认值、验证器和空值处理。
///
/// ## 样式示例
///
/// ```
/// ┌─────────────────────────────────────┐
/// │ Enter your name:                    │
/// │ > John Doe                          │
/// │                                     │
/// │ [Press Enter to confirm]            │
/// └─────────────────────────────────────┘
/// ```
///
/// 带验证器时的错误提示：
/// ```
/// ┌─────────────────────────────────────┐
/// │ Enter age:                          │
/// │ > abc                               │
/// │ ❌ Please enter a valid number      │
/// │                                     │
/// └─────────────────────────────────────┘
/// ```
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::dialog::InputDialog;
///
/// let name = InputDialog::new("Enter your name")
///     .with_default("John Doe")
///     .prompt()?;
/// ```
pub struct InputDialog {
    prompt: String,
    default: Option<String>,
    validator: Option<ValidatorFn>,
    allow_empty: bool,
}

impl InputDialog {
    /// 创建新的输入对话框
    ///
    /// # 参数
    ///
    /// * `prompt` - 提示信息
    ///
    /// # 返回
    ///
    /// 返回 `InputDialog` 实例
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            default: None,
            validator: None,
            allow_empty: false,
        }
    }

    /// 设置默认值
    ///
    /// # 参数
    ///
    /// * `default` - 默认值
    ///
    /// # 返回
    ///
    /// 返回 `Self` 以支持链式调用
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// 设置验证器
    ///
    /// # 参数
    ///
    /// * `validator` - 验证函数，返回 `Ok(())` 表示有效，`Err(String)` 表示错误信息
    ///
    /// # 返回
    ///
    /// 返回 `Self` 以支持链式调用
    pub fn with_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        self.validator = Some(Arc::new(validator));
        self
    }

    /// 允许空值
    ///
    /// # 参数
    ///
    /// * `allow` - 是否允许空值（默认：false）
    ///
    /// # 返回
    ///
    /// 返回 `Self` 以支持链式调用
    pub fn allow_empty(mut self, allow: bool) -> Self {
        self.allow_empty = allow;
        self
    }

    /// 显示对话框并获取用户输入
    ///
    /// # 返回
    ///
    /// 返回用户输入的字符串
    ///
    /// # 错误
    ///
    /// 如果用户取消输入或验证失败，返回错误
    pub fn prompt(self) -> Result<String> {
        let mut text = Text::new(&self.prompt);

        // 设置默认值
        if let Some(ref default) = self.default {
            text = text.with_default(default);
        }

        // 设置验证器
        if let Some(validator) = self.validator {
            let allow_empty = self.allow_empty;
            let validator_clone = validator.clone();
            text = text.with_validator(
                move |input: &str| -> Result<Validation, Box<dyn Error + Send + Sync>> {
                    // 如果允许空值且输入为空，直接通过
                    if allow_empty && input.trim().is_empty() {
                        return Ok(Validation::Valid);
                    }
                    // 否则使用验证器
                    match validator_clone(input) {
                        Ok(()) => Ok(Validation::Valid),
                        Err(msg) => Ok(Validation::Invalid(msg.into())),
                    }
                },
            );
        } else if !self.allow_empty {
            // 如果没有验证器但不允许空值，添加默认验证
            text = text.with_validator(
                |input: &str| -> Result<Validation, Box<dyn Error + Send + Sync>> {
                    if input.trim().is_empty() {
                        Ok(Validation::Invalid("Input cannot be empty".into()))
                    } else {
                        Ok(Validation::Valid)
                    }
                },
            );
        } else if self.allow_empty {
            // 如果允许空值，添加一个总是通过的验证器
            text = text.with_validator(
                |_input: &str| -> Result<Validation, Box<dyn Error + Send + Sync>> {
                    Ok(Validation::Valid)
                },
            );
        }

        text.prompt()
            .map_err(|e| match e {
                InquireError::OperationCanceled => {
                    anyhow::anyhow!("Operation cancelled by user")
                }
                _ => anyhow::anyhow!("Input error: {}", e),
            })
            .map(|s| s.trim().to_string())
    }
}

/// 单选对话框
///
/// 提供单选功能，从选项列表中选择一个选项。
///
/// ## 样式示例
///
/// ```
/// ┌─────────────────────────────────────┐
/// │ Choose an option:                    │
/// │                                     │
/// │   > Option 1                        │ ← 当前选中（高亮）
/// │     Option 2                        │
/// │     Option 3                        │
/// │                                     │
/// │ [↑↓: Move, Enter: Select, Esc: Cancel] │
/// └─────────────────────────────────────┘
/// ```
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::dialog::SelectDialog;
///
/// let options = vec!["Option 1", "Option 2", "Option 3"];
/// let selected = SelectDialog::new("Choose an option", options)
///     .with_default(0)
///     .prompt()?;
/// ```
pub struct SelectDialog<T> {
    prompt: String,
    options: Vec<T>,
    default: Option<usize>,
}

impl<T> SelectDialog<T>
where
    T: std::fmt::Display,
{
    /// 创建新的单选对话框
    ///
    /// # 参数
    ///
    /// * `prompt` - 提示信息
    /// * `options` - 选项列表
    ///
    /// # 返回
    ///
    /// 返回 `SelectDialog` 实例
    pub fn new(prompt: impl Into<String>, options: Vec<T>) -> Self {
        Self {
            prompt: prompt.into(),
            options,
            default: None,
        }
    }

    /// 设置默认选项索引
    ///
    /// # 参数
    ///
    /// * `index` - 默认选项的索引
    ///
    /// # 返回
    ///
    /// 返回 `Self` 以支持链式调用
    pub fn with_default(mut self, index: usize) -> Self {
        self.default = Some(index);
        self
    }

    /// 显示对话框并获取用户选择
    ///
    /// # 返回
    ///
    /// 返回用户选中的选项（值的所有权）
    ///
    /// # 错误
    ///
    /// 如果用户取消选择，返回错误
    pub fn prompt(self) -> Result<T> {
        if self.options.is_empty() {
            anyhow::bail!("No options available");
        }

        let mut select = Select::new(&self.prompt, self.options);

        // 设置默认值
        if let Some(default_idx) = self.default {
            select = select.with_starting_cursor(default_idx);
        }

        select.prompt().map_err(|e| match e {
            InquireError::OperationCanceled => {
                anyhow::anyhow!("Operation cancelled by user")
            }
            _ => anyhow::anyhow!("Selection error: {}", e),
        })
    }
}

/// 多选对话框
///
/// 提供多选功能，从选项列表中选择多个选项。
///
/// ## 样式示例
///
/// ```
/// ┌─────────────────────────────────────┐
/// │ Choose options:                      │
/// │                                     │
/// │   > [✓] Option 1                    │ ← 已选中
/// │     [ ] Option 2                    │
/// │     [✓] Option 3                    │ ← 已选中
/// │                                     │
/// │ [↑↓: Move, Space: Toggle, Enter: Confirm, Esc: Cancel] │
/// └─────────────────────────────────────┘
/// ```
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::dialog::MultiSelectDialog;
///
/// let options = vec!["Option 1", "Option 2", "Option 3"];
/// let selected = MultiSelectDialog::new("Choose options", options)
///     .prompt()?;
/// // selected 是 Vec<&str>，包含选中的选项
/// ```
pub struct MultiSelectDialog<T> {
    prompt: String,
    options: Vec<T>,
    default: Option<Vec<usize>>,
}

impl<T> MultiSelectDialog<T>
where
    T: std::fmt::Display,
{
    /// 创建新的多选对话框
    ///
    /// # 参数
    ///
    /// * `prompt` - 提示信息
    /// * `options` - 选项列表
    ///
    /// # 返回
    ///
    /// 返回 `MultiSelectDialog` 实例
    pub fn new(prompt: impl Into<String>, options: Vec<T>) -> Self {
        Self {
            prompt: prompt.into(),
            options,
            default: None,
        }
    }

    /// 设置默认选中的选项索引
    ///
    /// # 参数
    ///
    /// * `indices` - 默认选中的选项索引列表
    ///
    /// # 返回
    ///
    /// 返回 `Self` 以支持链式调用
    pub fn with_default(mut self, indices: Vec<usize>) -> Self {
        self.default = Some(indices);
        self
    }

    /// 显示对话框并获取用户选择
    ///
    /// # 返回
    ///
    /// 返回用户选中的选项列表
    ///
    /// # 错误
    ///
    /// 如果用户取消选择，返回错误
    pub fn prompt(self) -> Result<Vec<T>> {
        if self.options.is_empty() {
            anyhow::bail!("No options available");
        }

        let mut multi_select = MultiSelect::new(&self.prompt, self.options);

        // 设置默认值
        if let Some(ref default_indices) = self.default {
            multi_select = multi_select.with_default(default_indices.as_slice());
        }

        multi_select.prompt().map_err(|e| match e {
            InquireError::OperationCanceled => {
                anyhow::anyhow!("Operation cancelled by user")
            }
            _ => anyhow::anyhow!("Multi-selection error: {}", e),
        })
    }
}

/// 确认对话框
///
/// 提供确认功能，用于获取用户的 yes/no 选择。
///
/// ## 样式示例
///
/// 默认值为 true 时：
/// ```
/// ┌─────────────────────────────────────┐
/// │ Continue? (Y/n)                     │
/// │ > Yes                               │ ← 默认选中
/// │   No                                │
/// │                                     │
/// │ [Y: Yes, n: No, Enter: Confirm]     │
/// └─────────────────────────────────────┘
/// ```
///
/// 默认值为 false 时：
/// ```
/// ┌─────────────────────────────────────┐
/// │ This operation cannot be undone.    │
/// │ Continue? (y/N)                     │
/// │   Yes                               │
/// │ > No                                │ ← 默认选中
/// │                                     │
/// │ [y: Yes, N: No, Enter: Confirm]     │
/// └─────────────────────────────────────┘
/// ```
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::dialog::ConfirmDialog;
///
/// // 简单确认
/// let confirmed = ConfirmDialog::new("Continue?")
///     .with_default(true)
///     .prompt()?;
///
/// // 取消时返回错误
/// ConfirmDialog::new("This operation cannot be undone. Continue?")
///     .with_default(false)
///     .with_cancel_message("Operation cancelled.")
///     .prompt()?;
/// ```
pub struct ConfirmDialog {
    prompt: String,
    default: Option<bool>,
    cancel_message: Option<String>,
}

impl ConfirmDialog {
    /// 创建新的确认对话框
    ///
    /// # 参数
    ///
    /// * `prompt` - 提示信息
    ///
    /// # 返回
    ///
    /// 返回 `ConfirmDialog` 实例
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            default: None,
            cancel_message: None,
        }
    }

    /// 设置默认值
    ///
    /// # 参数
    ///
    /// * `default` - 默认选择（true 表示默认确认，false 表示默认取消）
    ///
    /// # 返回
    ///
    /// 返回 `Self` 以支持链式调用
    pub fn with_default(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }

    /// 设置取消消息
    ///
    /// 如果设置了取消消息，当用户取消时，会返回错误而不是 `Ok(false)`。
    ///
    /// # 参数
    ///
    /// * `message` - 取消时的错误消息
    ///
    /// # 返回
    ///
    /// 返回 `Self` 以支持链式调用
    pub fn with_cancel_message(mut self, message: impl Into<String>) -> Self {
        self.cancel_message = Some(message.into());
        self
    }

    /// 显示对话框并获取用户确认
    ///
    /// # 返回
    ///
    /// - 用户确认：返回 `Ok(true)`
    /// - 用户取消且设置了 `cancel_message`：返回错误
    /// - 用户取消且未设置 `cancel_message`：返回 `Ok(false)`
    ///
    /// # 错误
    ///
    /// 如果设置了 `cancel_message` 且用户取消，返回错误
    pub fn prompt(self) -> Result<bool> {
        let mut confirm = Confirm::new(&self.prompt);

        // 设置默认值
        if let Some(default) = self.default {
            confirm = confirm.with_default(default);
        }

        let confirmed = confirm.prompt().map_err(|e| match e {
            InquireError::OperationCanceled => {
                if let Some(ref msg) = self.cancel_message {
                    anyhow::anyhow!("{}", msg)
                } else {
                    anyhow::anyhow!("Operation cancelled by user")
                }
            }
            _ => anyhow::anyhow!("Confirmation error: {}", e),
        })?;

        // 如果用户取消且设置了取消消息，返回错误
        if !confirmed && self.cancel_message.is_some() {
            anyhow::bail!("{}", self.cancel_message.unwrap());
        }

        Ok(confirmed)
    }
}
