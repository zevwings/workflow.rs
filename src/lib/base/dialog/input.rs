use anyhow::Result;
use inquire::{error::InquireError, validator::Validation, Text};
use std::error::Error;
use std::sync::Arc;

use crate::base::dialog::types::ValidatorFn;

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
/// use workflow::base::dialog::InputDialog;
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
