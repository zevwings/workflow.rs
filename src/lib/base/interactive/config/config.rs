//! 提示功能配置定义

use std::sync::Arc;

/// 格式化函数的类型别名（单参数）
type FormatFn = Arc<dyn Fn(&str) -> String + Send + Sync>;
/// 格式化函数的类型别名（无参数）
type FormatFnNoArg = Arc<dyn Fn() -> String + Send + Sync>;
/// 格式化函数的类型别名（双参数）
type FormatFnTwoArgs = Arc<dyn Fn(&str, &str) -> String + Send + Sync>;

/// 提示功能的通用配置
/// 用于 select、multiselect、confirm、form 等交互式提示功能
#[derive(Clone)]
pub struct PromptConfig {
    /// 格式化提示消息的函数
    pub format_prompt: Option<FormatFn>,
    /// 格式化答案的函数
    pub format_answer: Option<FormatFn>,
    /// 格式化错误消息的函数（用于输入验证错误显示）
    pub format_error: Option<FormatFn>,
    /// 格式化提示信息（如操作说明）的函数
    pub format_hint: Option<FormatFn>,
    /// 格式化问题前缀 "? " 的函数
    pub format_question_prefix: Option<FormatFnNoArg>,
    /// 格式化答案前缀 "> " 的函数
    pub format_answer_prefix: Option<FormatFnNoArg>,
    /// 格式化完成后显示的 title 的函数
    /// 参数: originalMessage - 原始的提示消息, resultValue - 用户输入/选择的值
    /// 返回: 格式化后的 title 文本
    /// 如果为 None，则使用原始的 message
    pub format_result_title: Option<FormatFnTwoArgs>,
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptConfig {
    /// 创建新的空配置
    pub fn new() -> Self {
        Self {
            format_prompt: None,
            format_answer: None,
            format_error: None,
            format_hint: None,
            format_question_prefix: None,
            format_answer_prefix: None,
            format_result_title: None,
        }
    }

    /// 合并配置，将 base 和 override 合并，override 中的非 None 字段会覆盖 base
    pub fn merge(base: &Self, override_config: &Self) -> Self {
        Self {
            format_prompt: override_config
                .format_prompt
                .clone()
                .or_else(|| base.format_prompt.clone()),
            format_answer: override_config
                .format_answer
                .clone()
                .or_else(|| base.format_answer.clone()),
            format_error: override_config
                .format_error
                .clone()
                .or_else(|| base.format_error.clone()),
            format_hint: override_config.format_hint.clone().or_else(|| base.format_hint.clone()),
            format_question_prefix: override_config
                .format_question_prefix
                .clone()
                .or_else(|| base.format_question_prefix.clone()),
            format_answer_prefix: override_config
                .format_answer_prefix
                .clone()
                .or_else(|| base.format_answer_prefix.clone()),
            format_result_title: override_config
                .format_result_title
                .clone()
                .or_else(|| base.format_result_title.clone()),
        }
    }

    /// 填充配置的默认值
    /// 如果 config 中的某个字段为 None，则使用 default_config 中对应的字段
    pub fn fill_defaults(config: &Self, default_config: &Self) -> Self {
        Self {
            format_prompt: config
                .format_prompt
                .clone()
                .or_else(|| default_config.format_prompt.clone()),
            format_answer: config
                .format_answer
                .clone()
                .or_else(|| default_config.format_answer.clone()),
            format_error: config
                .format_error
                .clone()
                .or_else(|| default_config.format_error.clone()),
            format_hint: config.format_hint.clone().or_else(|| default_config.format_hint.clone()),
            format_question_prefix: config
                .format_question_prefix
                .clone()
                .or_else(|| default_config.format_question_prefix.clone()),
            format_answer_prefix: config
                .format_answer_prefix
                .clone()
                .or_else(|| default_config.format_answer_prefix.clone()),
            // FormatResultTitle 不填充默认值，保持为 None 表示使用原始 message
            format_result_title: config.format_result_title.clone(),
        }
    }
}

/// 为配置添加或覆盖 FormatResultTitle 的函数
/// 如果 result_title 为空字符串，返回原始配置
/// 否则返回新配置，其中 FormatResultTitle 返回固定的 result_title 字符串
pub fn with_result_title(config: PromptConfig, result_title: String) -> PromptConfig {
    if result_title.is_empty() {
        return config;
    }

    let title_str = result_title.clone();
    let mut new_config = config;
    new_config.format_result_title = Some(Arc::new(move |_original: &str, _result: &str| {
        title_str.clone()
    }));
    new_config
}
