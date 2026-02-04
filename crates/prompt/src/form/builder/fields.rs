//! 表单字段和组添加方法

use crate::form::builder::group_builder::GroupBuilder;
use crate::form::field::{
    ConfirmFormField, FieldType, FormField, InputFormField, MultiSelectFormField, NestedFormField,
    PasswordFormField, SelectFormField,
};
use crate::form::types::{FormGroup, GroupConfig};
use crate::form::FormBuilder;

impl FormBuilder {
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

    #[cfg(test)]
    pub(crate) fn get_fields_count(&self) -> usize {
        self.fields.len()
    }

    #[cfg(test)]
    pub(crate) fn get_groups_count(&self) -> usize {
        self.groups.len()
    }

    /// 添加表单组
    ///
    /// 使用 `GroupConfig` 配置组的行为和显示选项。
    ///
    /// # 参数
    ///
    /// * `id` - 组的唯一标识符
    /// * `builder` - 构建组内步骤的闭包
    /// * `config` - 组配置（使用 `GroupConfig::required()` 或 `GroupConfig::optional()` 创建）
    pub fn add_group<F>(mut self, id: impl Into<String>, builder: F, config: GroupConfig) -> Self
    where
        F: FnOnce(GroupBuilder) -> GroupBuilder,
    {
        let group_id = id.into();
        let group_builder = GroupBuilder::new(&group_id);
        let built = builder(group_builder);

        let group = FormGroup {
            title: config.title,
            description: config.description,
            optional: config.optional,
            default_enabled: config.default_enabled,
            steps: built.into_steps(),
        };

        self.groups.push(group);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_confirm_field() {
        let config = ConfirmFormField::new("agree", "Do you agree?").default(true);
        let builder = FormBuilder::new().add_confirm(config);

        assert_eq!(builder.get_fields_count(), 1);
        let field = &builder.fields[0];
        assert_eq!(field.key, "agree");
        assert_eq!(field.prompt, "Do you agree?");
        assert!(matches!(field.field_type, FieldType::Confirm));
    }

    #[test]
    fn test_add_input_field() {
        let config = InputFormField::new("name", "Enter name").default("John");
        let builder = FormBuilder::new().add_input(config);

        assert_eq!(builder.get_fields_count(), 1);
        let field = &builder.fields[0];
        assert_eq!(field.key, "name");
        assert_eq!(field.prompt, "Enter name");
        assert!(matches!(field.field_type, FieldType::Input));
    }

    #[test]
    fn test_add_password_field() {
        let config = PasswordFormField::new("password", "Enter password");
        let builder = FormBuilder::new().add_password(config);

        assert_eq!(builder.get_fields_count(), 1);
        let field = &builder.fields[0];
        assert_eq!(field.key, "password");
        assert!(matches!(field.field_type, FieldType::Password));
    }

    #[test]
    fn test_add_select_field() {
        let options = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let config = SelectFormField::new("choice", "Select one", options.clone()).default(1);
        let builder = FormBuilder::new().add_select(config);

        assert_eq!(builder.get_fields_count(), 1);
        let field = &builder.fields[0];
        assert_eq!(field.key, "choice");
        assert!(matches!(field.field_type, FieldType::Select));
        assert_eq!(field.options, options);
        assert_eq!(field.default_index, Some(1));
    }

    #[test]
    fn test_add_multiselect_field() {
        let options = vec!["X".to_string(), "Y".to_string(), "Z".to_string()];
        let config = MultiSelectFormField::new("choices", "Select multiple", options.clone())
            .default(vec![0, 2]);
        let builder = FormBuilder::new().add_multiselect(config);

        assert_eq!(builder.get_fields_count(), 1);
        let field = &builder.fields[0];
        assert_eq!(field.key, "choices");
        assert!(matches!(field.field_type, FieldType::MultiSelect));
        assert_eq!(field.options, options);
        assert_eq!(field.default_selected, vec![0, 2]);
    }

    #[test]
    fn test_add_nested_form_field() {
        let nested = FormBuilder::new().add_input(InputFormField::new("inner", "Inner field"));
        let config = NestedFormField::new("nested", "Nested form", nested);
        let builder = FormBuilder::new().add_form(config);

        assert_eq!(builder.get_fields_count(), 1);
        let field = &builder.fields[0];
        assert_eq!(field.key, "nested");
        assert!(matches!(field.field_type, FieldType::Form));
        assert!(field.nested_form.is_some());
    }

    #[test]
    fn test_add_multiple_fields() {
        let builder = FormBuilder::new()
            .add_input(InputFormField::new("name", "Name"))
            .add_input(InputFormField::new("email", "Email"))
            .add_confirm(ConfirmFormField::new("agree", "Agree?"));

        assert_eq!(builder.get_fields_count(), 3);
        assert_eq!(builder.fields[0].key, "name");
        assert_eq!(builder.fields[1].key, "email");
        assert_eq!(builder.fields[2].key, "agree");
    }

    #[test]
    fn test_add_group() {
        let config = GroupConfig::required().with_title("Group 1");
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.add_step(|s| s.add_input(InputFormField::new("field1", "Field 1"))),
            config,
        );

        assert_eq!(builder.get_groups_count(), 1);
        assert!(builder.has_groups());
    }

    #[test]
    fn test_field_with_result_title() {
        let config = InputFormField::new("name", "Enter name").result_title("Your Name");
        let builder = FormBuilder::new().add_input(config);

        let field = &builder.fields[0];
        assert_eq!(field.result_title, Some("Your Name".to_string()));
    }

    #[test]
    fn test_field_with_condition() {
        let condition: crate::form::field::Condition = Box::new(|_| true);
        let config = InputFormField::new("optional", "Optional field").condition(condition);
        let builder = FormBuilder::new().add_input(config);

        let field = &builder.fields[0];
        assert!(field.condition.is_some());
    }
}
