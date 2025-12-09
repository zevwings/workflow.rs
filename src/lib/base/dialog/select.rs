use anyhow::Result;
use inquire::{error::InquireError, Select};

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
/// use workflow::base::dialog::SelectDialog;
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
