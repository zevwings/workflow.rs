//! 单选字段配置

use crate::form::field::types::Condition;

/// 单选字段配置
pub struct SelectFormField {
    /// 字段键名（用于结果映射）
    pub key: String,
    /// 提示消息
    pub prompt: String,
    /// 选项列表
    pub options: Vec<String>,
    /// 默认选中的索引
    pub default_index: usize,
    /// 输入完成后显示的 title（可选，字段级别）
    pub result_title: Option<String>,
    /// 条件函数（可选，基于前面字段的值决定是否执行）
    pub condition: Option<Condition>,
}

impl SelectFormField {
    /// 创建新的单选字段
    pub fn new(key: impl Into<String>, prompt: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            key: key.into(),
            prompt: prompt.into(),
            options,
            default_index: 0,
            result_title: None,
            condition: None,
        }
    }

    /// 设置默认选中的索引
    pub fn default(mut self, index: usize) -> Self {
        self.default_index = index;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_options() -> Vec<String> {
        vec![
            "Option A".to_string(),
            "Option B".to_string(),
            "Option C".to_string(),
        ]
    }

    #[test]
    fn test_select_form_field_new() {
        let options = sample_options();
        let field = SelectFormField::new("choice", "Select an option", options.clone());
        assert_eq!(field.key, "choice");
        assert_eq!(field.prompt, "Select an option");
        assert_eq!(field.options, options);
        assert_eq!(field.default_index, 0);
        assert!(field.result_title.is_none());
        assert!(field.condition.is_none());
    }

    #[test]
    fn test_select_form_field_with_default_index() {
        let options = sample_options();
        let field = SelectFormField::new("choice", "Select", options).default(2);
        assert_eq!(field.default_index, 2);
    }

    #[test]
    fn test_select_form_field_with_result_title() {
        let options = sample_options();
        let field = SelectFormField::new("color", "Select color", options).result_title("Color");
        assert_eq!(field.result_title, Some("Color".to_string()));
    }

    #[test]
    fn test_select_form_field_with_condition() {
        let options = sample_options();
        let condition: Condition = Box::new(|_result| true);
        let field = SelectFormField::new("choice", "Select", options).condition(condition);
        assert!(field.condition.is_some());
    }

    #[test]
    fn test_select_form_field_builder_chain() {
        let options = sample_options();
        let field = SelectFormField::new("theme", "Select theme", options.clone())
            .default(1)
            .result_title("Theme");

        assert_eq!(field.key, "theme");
        assert_eq!(field.prompt, "Select theme");
        assert_eq!(field.options, options);
        assert_eq!(field.default_index, 1);
        assert_eq!(field.result_title, Some("Theme".to_string()));
    }

    #[test]
    fn test_select_form_field_empty_options() {
        let field = SelectFormField::new("empty", "Select", Vec::new());
        assert!(field.options.is_empty());
        assert_eq!(field.default_index, 0);
    }
}
