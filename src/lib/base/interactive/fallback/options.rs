//! Fallback 选项定义

use crate::base::interactive::config::PromptConfig;
use crate::base::interactive::error::Result;
use crate::base::interactive::terminal::Terminal;

/// 格式化选项列表的函数类型
type FormatOptionsFn = Box<dyn Fn(&mut dyn Terminal) -> Result<()> + Send + Sync>;
/// 结果显示函数的类型
type ResultDisplayFn<T> = Box<
    dyn Fn(
            &mut dyn Terminal,
            &str,
            &T,
            &dyn super::handler::FallbackHandler<T>,
            &str,
            &PromptConfig,
        ) -> Result<()>
        + Send
        + Sync,
>;

/// 类型安全的 fallback 执行选项
pub struct FallbackOptions<T> {
    /// 是否显示选项列表（用于 select/multiselect）
    pub show_options: bool,
    /// 格式化选项列表的函数（如果 show_options 为 true，可以提供）
    pub format_options: Option<FormatOptionsFn>,
    /// 输入提示文本（如 "请选择 (1-3): "）
    pub input_prompt: Option<String>,
    /// 结果显示函数（用于显示最终结果）
    /// 参数: terminal, prompt_msg, result, handler, original_message, config
    pub result_display: Option<ResultDisplayFn<T>>,
}

impl<T> Default for FallbackOptions<T> {
    fn default() -> Self {
        Self {
            show_options: false,
            format_options: None,
            input_prompt: None,
            result_display: None,
        }
    }
}

impl<T> FallbackOptions<T> {
    /// 创建默认的 fallback 选项
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置是否显示选项列表
    pub fn with_show_options(mut self, show: bool) -> Self {
        self.show_options = show;
        self
    }

    /// 设置格式化选项列表的函数
    pub fn with_format_options<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut dyn Terminal) -> Result<()> + Send + Sync + 'static,
    {
        self.format_options = Some(Box::new(f));
        self
    }

    /// 设置输入提示文本
    pub fn with_input_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.input_prompt = Some(prompt.into());
        self
    }

    /// 设置结果显示函数
    pub fn with_result_display<F>(mut self, f: F) -> Self
    where
        F: Fn(
                &mut dyn Terminal,
                &str,
                &T,
                &dyn super::handler::FallbackHandler<T>,
                &str,
                &PromptConfig,
            ) -> Result<()>
            + Send
            + Sync
            + 'static,
    {
        self.result_display = Some(Box::new(f));
        self
    }
}
