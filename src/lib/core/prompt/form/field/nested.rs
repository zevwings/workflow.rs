//! 嵌套表单字段配置

use super::types::Condition;

/// 嵌套表单字段配置
pub struct NestedFormField {
    /// 字段键名（用于结果映射）
    pub key: String,
    /// 提示消息
    pub prompt: String,
    /// 嵌套表单
    pub nested_form: crate::prompt::form::FormBuilder,
    /// 输入完成后显示的 title（可选，字段级别）
    pub result_title: Option<String>,
    /// 条件函数（可选，基于前面字段的值决定是否执行）
    pub condition: Option<Condition>,
}

impl NestedFormField {
    /// 创建新的嵌套表单字段
    pub fn new(
        key: impl Into<String>,
        prompt: impl Into<String>,
        nested_form: crate::prompt::form::FormBuilder,
    ) -> Self {
        Self {
            key: key.into(),
            prompt: prompt.into(),
            nested_form,
            result_title: None,
            condition: None,
        }
    }

    /// 设置输入完成后显示的 title
    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }
}
