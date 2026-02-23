//! 多选字段配置

use crate::form::field::types::Condition;

/// 多选字段配置
pub struct MultiSelectFormField {
    /// 字段键名（用于结果映射）
    pub key: String,
    /// 提示消息
    pub prompt: String,
    /// 选项列表
    pub options: Vec<String>,
    /// 默认选中的索引列表
    pub default_selected: Vec<usize>,
    /// 输入完成后显示的 title（可选，字段级别）
    pub result_title: Option<String>,
    /// 条件函数（可选，基于前面字段的值决定是否执行）
    pub condition: Option<Condition>,
}

impl MultiSelectFormField {
    /// 创建新的多选字段
    pub fn new(key: impl Into<String>, prompt: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            key: key.into(),
            prompt: prompt.into(),
            options,
            default_selected: Vec::new(),
            result_title: None,
            condition: None,
        }
    }

    /// 设置默认选中的索引列表
    pub fn default(mut self, indices: Vec<usize>) -> Self {
        self.default_selected = indices;
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
            "Feature A".to_string(),
            "Feature B".to_string(),
            "Feature C".to_string(),
            "Feature D".to_string(),
        ]
    }

    #[test]
    fn test_multiselect_form_field_new() {
        let options = sample_options();
        let field = MultiSelectFormField::new("features", "Select features", options.clone());
        assert_eq!(field.key, "features");
        assert_eq!(field.prompt, "Select features");
        assert_eq!(field.options, options);
        assert!(field.default_selected.is_empty());
        assert!(field.result_title.is_none());
        assert!(field.condition.is_none());
    }

    #[test]
    fn test_multiselect_form_field_with_default_selected() {
        let options = sample_options();
        let field = MultiSelectFormField::new("features", "Select", options).default(vec![0, 2, 3]);
        assert_eq!(field.default_selected, vec![0, 2, 3]);
    }

    #[test]
    fn test_multiselect_form_field_with_empty_default() {
        let options = sample_options();
        let field = MultiSelectFormField::new("features", "Select", options).default(vec![]);
        assert!(field.default_selected.is_empty());
    }

    #[test]
    fn test_multiselect_form_field_with_result_title() {
        let options = sample_options();
        let field =
            MultiSelectFormField::new("features", "Select", options).result_title("Features");
        assert_eq!(field.result_title, Some("Features".to_string()));
    }

    #[test]
    fn test_multiselect_form_field_with_condition() {
        let options = sample_options();
        let condition: Condition = Box::new(|_result| true);
        let field = MultiSelectFormField::new("features", "Select", options).condition(condition);
        assert!(field.condition.is_some());
    }

    #[test]
    fn test_multiselect_form_field_builder_chain() {
        let options = sample_options();
        let field = MultiSelectFormField::new("toppings", "Select toppings", options.clone())
            .default(vec![0, 1])
            .result_title("Selected Toppings");

        assert_eq!(field.key, "toppings");
        assert_eq!(field.prompt, "Select toppings");
        assert_eq!(field.options, options);
        assert_eq!(field.default_selected, vec![0, 1]);
        assert_eq!(field.result_title, Some("Selected Toppings".to_string()));
    }

    #[test]
    fn test_multiselect_form_field_empty_options() {
        let field = MultiSelectFormField::new("empty", "Select", Vec::new());
        assert!(field.options.is_empty());
        assert!(field.default_selected.is_empty());
    }

    #[test]
    fn test_multiselect_form_field_single_selection() {
        let options = sample_options();
        let field = MultiSelectFormField::new("single", "Select", options).default(vec![1]);
        assert_eq!(field.default_selected, vec![1]);
    }

    #[test]
    fn test_multiselect_form_field_all_selected() {
        let options = sample_options();
        let field =
            MultiSelectFormField::new("all", "Select all", options).default(vec![0, 1, 2, 3]);
        assert_eq!(field.default_selected, vec![0, 1, 2, 3]);
    }
}
