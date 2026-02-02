//! 确认字段配置

use crate::form::field::types::Condition;

/// 确认字段配置
pub struct ConfirmFormField {
    /// 字段键名（用于结果映射）
    pub key: String,
    /// 提示消息
    pub prompt: String,
    /// 默认值
    pub default_value: bool,
    /// 输入完成后显示的 title（可选，字段级别）
    pub result_title: Option<String>,
    /// 条件函数（可选，基于前面字段的值决定是否执行）
    pub condition: Option<Condition>,
}

impl ConfirmFormField {
    /// 创建新的确认字段
    pub fn new(key: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            prompt: prompt.into(),
            default_value: false,
            result_title: None,
            condition: None,
        }
    }

    /// 设置默认值
    pub fn default(mut self, value: bool) -> Self {
        self.default_value = value;
        self
    }

    /// 设置条件函数
    pub fn condition(mut self, condition: Condition) -> Self {
        self.condition = Some(condition);
        self
    }

    /// 设置输入完成后显示的 title
    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }
}
