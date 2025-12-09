use anyhow::Result;
use inquire::{error::InquireError, Confirm};

/// 确认对话框
///
/// 提供确认功能，用于获取用户的 yes/no 选择。
///
/// ## 样式示例
///
/// 默认值为 true 时：
/// ```text
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
/// ```text
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
/// use workflow::base::dialog::ConfirmDialog;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
/// # Ok(())
/// # }
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
