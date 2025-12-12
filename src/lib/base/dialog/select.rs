use color_eyre::{eyre::eyre, Result};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use inquire::{error::InquireError, Select};

/// 单选对话框
///
/// 提供单选功能，从选项列表中选择一个选项。
///
/// ## 样式示例
///
/// ```text
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
/// ## 基本用法
///
/// ```rust,no_run
/// use workflow::base::dialog::SelectDialog;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let options = vec!["Option 1", "Option 2", "Option 3"];
/// let selected = SelectDialog::new("Choose an option", options)
///     .with_default(0)
///     .prompt()?;
/// # Ok(())
/// # }
/// ```
///
/// ## 模糊匹配
///
/// SelectDialog 默认启用模糊匹配过滤器（使用 SkimMatcherV2），支持快速查找选项：
///
/// ```rust,no_run
/// use workflow::base::dialog::SelectDialog;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let branches = vec![
///     "feature/user-authentication",
///     "feature/payment-integration",
///     "bugfix/login-error",
///     "refactoring/api-structure",
/// ];
/// let selected = SelectDialog::new("选择分支", branches)
///     .prompt()?;
/// // 用户可以输入 "feat" 来匹配所有 feature 分支
/// // 或输入 "fb" 来匹配 "feature/bugfix" 等
/// # Ok(())
/// # }
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
            color_eyre::eyre::bail!("No options available");
        }

        let mut select = Select::new(&self.prompt, self.options);

        // 设置默认值
        if let Some(default_idx) = self.default {
            select = select.with_starting_cursor(default_idx);
        }

        // 应用模糊匹配过滤器（默认启用）
        // 定义模糊匹配 scorer 函数
        // 注意：inquire 的 with_scorer 需要函数引用，且返回 Option<i64>
        // 参数类型是 (&str, &T, &str, usize) -> Option<i64>
        fn fuzzy_scorer<T: std::fmt::Display>(
            input: &str,
            option: &T,
            _display: &str,
            _index: usize,
        ) -> Option<i64> {
            // 如果输入为空，显示所有选项（返回高分）
            if input.is_empty() {
                return Some(1000);
            }

            // 创建匹配器并执行模糊匹配
            // 注意：每次调用都创建新的 matcher，但这是轻量级操作
            let matcher = SkimMatcherV2::default();
            let option_str = option.to_string();

            // 使用模糊匹配计算分数
            // fuzzy_match 返回 Option<usize>，转换为 i64
            matcher.fuzzy_match(&option_str, input)
        }

        select = select.with_scorer(&fuzzy_scorer);

        select.prompt().map_err(|e| match e {
            InquireError::OperationCanceled => {
                eyre!("Operation cancelled by user")
            }
            _ => eyre!("Selection error: {}", e),
        })
    }
}
