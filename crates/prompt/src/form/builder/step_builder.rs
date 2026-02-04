//! Step 构建器，用于在 Step 内构建字段

use crate::form::field::{
    ConfirmFormField, FieldType, FormField, InputFormField, MultiSelectFormField, NestedFormField,
    PasswordFormField, SelectFormField,
};

/// Step 构建器
///
/// 用于在 Step 内构建字段，提供便捷的方法来添加各种类型的字段。
pub struct StepBuilder {
    fields: Vec<FormField>,
}

impl StepBuilder {
    /// 创建新的 Step 构建器
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// 添加确认字段
    pub fn add_confirm(mut self, config: ConfirmFormField) -> Self {
        self.fields.push(FormField {
            key: config.key,
            field_type: FieldType::Confirm,
            prompt: config.prompt,
            default_value: Some(Box::new(config.default_value)),
            validator: None,
            condition: config.condition,
            result_title: config.result_title,
            nested_form: None,
            options: Vec::new(),
            default_index: None,
            default_selected: Vec::new(),
        });
        self
    }

    /// 添加输入字段
    pub fn add_input(mut self, config: InputFormField) -> Self {
        self.fields.push(FormField {
            key: config.key,
            field_type: FieldType::Input,
            prompt: config.prompt,
            default_value: Some(Box::new(config.default_value)),
            validator: config.validator,
            condition: config.condition,
            result_title: config.result_title,
            nested_form: None,
            options: Vec::new(),
            default_index: None,
            default_selected: Vec::new(),
        });
        self
    }

    /// 添加密码字段
    pub fn add_password(mut self, config: PasswordFormField) -> Self {
        self.fields.push(FormField {
            key: config.key,
            field_type: FieldType::Password,
            prompt: config.prompt,
            default_value: Some(Box::new(config.default_value)),
            validator: config.validator,
            condition: config.condition,
            result_title: config.result_title,
            nested_form: None,
            options: Vec::new(),
            default_index: None,
            default_selected: Vec::new(),
        });
        self
    }

    /// 添加单选字段
    pub fn add_select(mut self, config: SelectFormField) -> Self {
        self.fields.push(FormField {
            key: config.key,
            field_type: FieldType::Select,
            prompt: config.prompt,
            default_value: None,
            validator: None,
            condition: config.condition,
            result_title: config.result_title,
            nested_form: None,
            options: config.options,
            default_index: Some(config.default_index),
            default_selected: Vec::new(),
        });
        self
    }

    /// 添加多选字段
    pub fn add_multiselect(mut self, config: MultiSelectFormField) -> Self {
        self.fields.push(FormField {
            key: config.key,
            field_type: FieldType::MultiSelect,
            prompt: config.prompt,
            default_value: None,
            validator: None,
            condition: config.condition,
            result_title: config.result_title,
            nested_form: None,
            options: config.options,
            default_index: None,
            default_selected: config.default_selected,
        });
        self
    }

    /// 添加嵌套表单字段
    pub fn add_form(mut self, config: NestedFormField) -> Self {
        self.fields.push(FormField {
            key: config.key,
            field_type: FieldType::Form,
            prompt: config.prompt,
            default_value: None,
            validator: None,
            condition: config.condition,
            result_title: config.result_title,
            nested_form: Some(config.nested_form),
            options: Vec::new(),
            default_index: None,
            default_selected: Vec::new(),
        });
        self
    }

    /// 获取字段列表（内部使用）
    pub(crate) fn into_fields(self) -> Vec<FormField> {
        self.fields
    }
}

impl Default for StepBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::form::FormBuilder;

    #[test]
    fn test_step_builder_new() {
        let builder = StepBuilder::new();
        let fields = builder.into_fields();
        assert!(fields.is_empty());
    }

    #[test]
    fn test_step_builder_default() {
        let builder = StepBuilder::default();
        let fields = builder.into_fields();
        assert!(fields.is_empty());
    }

    #[test]
    fn test_step_builder_add_confirm() {
        let builder =
            StepBuilder::new().add_confirm(ConfirmFormField::new("confirm_key", "Do you agree?"));

        let fields = builder.into_fields();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "confirm_key");
        assert_eq!(fields[0].field_type, FieldType::Confirm);
        assert_eq!(fields[0].prompt, "Do you agree?");
    }

    #[test]
    fn test_step_builder_add_confirm_with_default() {
        let builder =
            StepBuilder::new().add_confirm(ConfirmFormField::new("key", "Question").default(true));

        let fields = builder.into_fields();
        assert_eq!(fields.len(), 1);

        let default_value = fields[0].default_value.as_ref().and_then(|v| v.downcast_ref::<bool>());
        assert_eq!(default_value, Some(&true));
    }

    #[test]
    fn test_step_builder_add_input() {
        let builder = StepBuilder::new().add_input(InputFormField::new("name", "Enter your name"));

        let fields = builder.into_fields();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "name");
        assert_eq!(fields[0].field_type, FieldType::Input);
        assert_eq!(fields[0].prompt, "Enter your name");
    }

    #[test]
    fn test_step_builder_add_input_with_default() {
        let builder =
            StepBuilder::new().add_input(InputFormField::new("name", "Name").default("John"));

        let fields = builder.into_fields();
        let default_value =
            fields[0].default_value.as_ref().and_then(|v| v.downcast_ref::<String>());
        assert_eq!(default_value, Some(&"John".to_string()));
    }

    #[test]
    fn test_step_builder_add_password() {
        let builder =
            StepBuilder::new().add_password(PasswordFormField::new("password", "Enter password"));

        let fields = builder.into_fields();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "password");
        assert_eq!(fields[0].field_type, FieldType::Password);
    }

    #[test]
    fn test_step_builder_add_select() {
        let options = vec!["Option A".to_string(), "Option B".to_string()];
        let builder = StepBuilder::new().add_select(SelectFormField::new(
            "choice",
            "Choose one",
            options.clone(),
        ));

        let fields = builder.into_fields();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "choice");
        assert_eq!(fields[0].field_type, FieldType::Select);
        assert_eq!(fields[0].options, options);
        assert_eq!(fields[0].default_index, Some(0));
    }

    #[test]
    fn test_step_builder_add_select_with_default_index() {
        let options = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let builder = StepBuilder::new()
            .add_select(SelectFormField::new("choice", "Choose", options).default(2));

        let fields = builder.into_fields();
        assert_eq!(fields[0].default_index, Some(2));
    }

    #[test]
    fn test_step_builder_add_multiselect() {
        let options = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let builder = StepBuilder::new().add_multiselect(MultiSelectFormField::new(
            "choices",
            "Select multiple",
            options.clone(),
        ));

        let fields = builder.into_fields();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "choices");
        assert_eq!(fields[0].field_type, FieldType::MultiSelect);
        assert_eq!(fields[0].options, options);
    }

    #[test]
    fn test_step_builder_add_multiselect_with_defaults() {
        let options = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let builder = StepBuilder::new().add_multiselect(
            MultiSelectFormField::new("choices", "Select", options).default(vec![0, 2]),
        );

        let fields = builder.into_fields();
        assert_eq!(fields[0].default_selected, vec![0, 2]);
    }

    #[test]
    fn test_step_builder_add_form() {
        let nested =
            FormBuilder::new().add_input(InputFormField::new("nested_field", "Nested input"));

        let builder =
            StepBuilder::new().add_form(NestedFormField::new("nested", "Nested Form", nested));

        let fields = builder.into_fields();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "nested");
        assert_eq!(fields[0].field_type, FieldType::Form);
        assert!(fields[0].nested_form.is_some());
    }

    #[test]
    fn test_step_builder_multiple_fields() {
        let builder = StepBuilder::new()
            .add_input(InputFormField::new("name", "Name"))
            .add_input(InputFormField::new("email", "Email"))
            .add_confirm(ConfirmFormField::new("agree", "Agree?"));

        let fields = builder.into_fields();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].key, "name");
        assert_eq!(fields[1].key, "email");
        assert_eq!(fields[2].key, "agree");
    }

    #[test]
    fn test_step_builder_with_condition() {
        let builder = StepBuilder::new().add_input(
            InputFormField::new("extra", "Extra field")
                .condition(Box::new(|result| result.get_bool("enabled"))),
        );

        let fields = builder.into_fields();
        assert_eq!(fields.len(), 1);
        assert!(fields[0].condition.is_some());
    }

    #[test]
    fn test_step_builder_with_result_title() {
        let builder = StepBuilder::new()
            .add_input(InputFormField::new("name", "Enter name").result_title("Your Name"));

        let fields = builder.into_fields();
        assert_eq!(fields[0].result_title, Some("Your Name".to_string()));
    }

    #[test]
    fn test_step_builder_chain_preserves_order() {
        let builder = StepBuilder::new()
            .add_confirm(ConfirmFormField::new("step1", "Step 1"))
            .add_input(InputFormField::new("step2", "Step 2"))
            .add_password(PasswordFormField::new("step3", "Step 3"))
            .add_select(SelectFormField::new(
                "step4",
                "Step 4",
                vec!["A".to_string()],
            ))
            .add_multiselect(MultiSelectFormField::new(
                "step5",
                "Step 5",
                vec!["B".to_string()],
            ));

        let fields = builder.into_fields();
        assert_eq!(fields.len(), 5);
        assert_eq!(fields[0].field_type, FieldType::Confirm);
        assert_eq!(fields[1].field_type, FieldType::Input);
        assert_eq!(fields[2].field_type, FieldType::Password);
        assert_eq!(fields[3].field_type, FieldType::Select);
        assert_eq!(fields[4].field_type, FieldType::MultiSelect);
    }
}
