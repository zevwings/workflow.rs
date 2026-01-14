//! 表单构建器

use crate::prompt::form::field::{
    ConfirmFormField, FieldType, FormField, InputFormField, MultiSelectFormField, NestedFormField,
    PasswordFormField, SelectFormField,
};
use crate::prompt::form::group::{FormGroup, GroupConfig};
use crate::prompt::form::group_builder::GroupBuilder;

/// 表单构建器（链式 API）
pub struct FormBuilder {
    /// 字段列表（用于简单模式，不使用 Group）
    fields: Vec<FormField>,
    /// 组列表（用于 Group/Step 模式）
    groups: Vec<FormGroup>,
    /// 表单标题
    title: Option<String>,
}

impl FormBuilder {
    /// 创建新的表单构建器
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            groups: Vec::new(),
            title: None,
        }
    }

    /// 设置表单标题
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
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
            id: group_id,
            title: config.title,
            description: config.description,
            optional: config.optional,
            default_enabled: config.default_enabled,
            steps: built.into_steps(),
        };

        self.groups.push(group);
        self
    }

    /// 获取字段列表（内部使用，用于简单模式）
    pub(crate) fn get_fields(&self) -> &[FormField] {
        &self.fields
    }

    /// 获取组列表（内部使用，用于 Group/Step 模式）
    pub(crate) fn get_groups(&self) -> &[FormGroup] {
        &self.groups
    }

    /// 检查是否使用 Group 模式
    pub(crate) fn has_groups(&self) -> bool {
        !self.groups.is_empty()
    }

    /// 获取表单标题
    pub fn get_title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// 执行表单并收集用户输入（兼容旧 API）
    ///
    /// 这个方法内部使用 `FormExecutor` 来执行表单。
    pub fn run(self) -> color_eyre::Result<crate::prompt::form::result::FormResult> {
        use crate::prompt::form::executor::FormExecutor;
        FormExecutor::new().execute(&self).map_err(|e| color_eyre::eyre::eyre!("{}", e))
    }
}

impl Default for FormBuilder {
    fn default() -> Self {
        Self::new()
    }
}
