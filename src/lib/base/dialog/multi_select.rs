use color_eyre::{eyre::eyre, Result};
use inquire::{error::InquireError, MultiSelect};

use crate::base::dialog::skip_config;

/// 多选对话框
///
/// 提供多选功能，从选项列表中选择多个选项。
///
/// ## 样式示例
///
/// ```text
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
/// use workflow::base::dialog::MultiSelectDialog;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let options = vec!["Option 1", "Option 2", "Option 3"];
/// let selected = MultiSelectDialog::new("Choose options", options)
///     .prompt()?;
/// // selected 是 Vec<&str>，包含选中的选项
/// # Ok(())
/// # }
/// ```
pub struct MultiSelectDialog<T> {
    prompt: String,
    options: Vec<T>,
    default: Option<Vec<usize>>,
}

impl<T> MultiSelectDialog<T>
where
    T: std::fmt::Display + Clone,
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
        // 检查 thread-local 配置（用于测试）
        if skip_config::DialogConfigManager::is_non_interactive() {
            if let Some(indices) = skip_config::DialogConfigManager::get_multi_select_indices() {
                let mut result = Vec::new();
                for index in indices {
                    if index < self.options.len() {
                        result.push(self.options[index].clone());
                    }
                }
                return Ok(result);
            }
            // 如果启用了非交互式模式但没有设置索引，使用默认索引或空列表
            if let Some(ref default_indices) = self.default {
                let mut result = Vec::new();
                for &index in default_indices {
                    if index < self.options.len() {
                        result.push(self.options[index].clone());
                    }
                }
                return Ok(result);
            }
            return Ok(Vec::new());
        }

        if self.options.is_empty() {
            color_eyre::eyre::bail!("No options available");
        }

        let mut multi_select = MultiSelect::new(&self.prompt, self.options);

        // 设置默认值
        if let Some(ref default_indices) = self.default {
            multi_select = multi_select.with_default(default_indices.as_slice());
        }

        multi_select.prompt().map_err(|e| match e {
            InquireError::OperationCanceled => {
                eyre!("Operation cancelled by user")
            }
            _ => eyre!("Multi-selection error: {}", e),
        })
    }
}
