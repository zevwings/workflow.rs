//! 表单构建器

use std::collections::HashMap;

use color_eyre::{eyre::eyre, Result};
use dialoguer::Password;

use crate::base::dialog::form::condition_evaluator::ConditionEvaluator;
use crate::base::dialog::form::group_builder::GroupBuilder;
use crate::base::dialog::form::types::{FormGroup, FormStep, GroupConfig, StepType};
use crate::base::dialog::form::{FieldDefaultValue, FormField, FormFieldType, FormResult};
use crate::base::dialog::{ConfirmDialog, InputDialog, SelectDialog};
use crate::{log_break, log_debug, log_message};

/// 表单构建器
///
/// 提供 Group 支持的表单构建器，可以将整个 setup 的所有内容封装为一个 form。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::dialog::{FormBuilder, GroupConfig};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // 必填组
/// let form_result = FormBuilder::new()
///     .add_group("jira", |g| {
///         g.step(|f| {
///             f.add_text("jira_email", "Jira email address").required()
///         })
///     }, GroupConfig::required())
    ///     // 可选组（带标题）
    ///     .add_group("llm", |g| {
    ///         g.step(|f| {
    ///             f.add_selection("llm_provider", "Select LLM provider", vec!["openai".into(), "deepseek".into()])
    ///         })
    ///     }, GroupConfig::optional()
///         .with_title("LLM/AI Configuration")
///         .with_default_enabled(true))
///     .run()?;
/// # Ok(())
/// # }
/// ```
pub struct FormBuilder {
    /// 表单组列表
    pub groups: Vec<FormGroup>,
}

impl FormBuilder {
    /// 创建新的统一表单构建器
    pub fn new() -> Self {
        Self { groups: Vec::new() }
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
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::base::dialog::{FormBuilder, GroupConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // 必填组
    /// FormBuilder::new()
    ///     .add_group("jira", |g| {
    ///         g.step(|f| {
    ///             f.add_text("jira_email", "Jira email address").required()
    ///         })
    ///     }, GroupConfig::required())
    ///     // 可选组（带标题）
    ///     .add_group("llm", |g| {
    ///         g.step(|f| {
    ///             f.add_selection("llm_provider", "...", vec!["openai".into(), "deepseek".into()])
    ///         })
    ///     }, GroupConfig::optional()
    ///         .with_title("LLM Configuration"))
    ///     // 可选组（带标题和描述）
    ///     .add_group("log", |g| {
    ///         g.step(|f| {
    ///             f.add_text("log_level", "Log level").required()
    ///         })
    ///     }, GroupConfig::optional()
    ///         .with_title("Log Configuration")
    ///         .with_description("Configure logging settings")
    ///         .with_default_enabled(false))
    ///     .run()?;
    /// # Ok(())
    /// # }
    /// ```
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

    /// 验证表单构建器的配置
    ///
    /// 检查组 ID 唯一性、步骤非空等。
    ///
    /// # 错误
    ///
    /// 如果验证失败，返回错误
    fn validate(&self) -> Result<()> {
        use std::collections::HashSet;

        // 检查组 ID 唯一性
        let mut group_ids = HashSet::new();
        for group in &self.groups {
            if !group_ids.insert(&group.id) {
                return Err(color_eyre::eyre::eyre!(
                    "Duplicate group ID: '{}'. Group IDs must be unique.",
                    group.id
                ));
            }
        }

        // 检查组是否有步骤
        for group in &self.groups {
            if group.steps.is_empty() {
                return Err(color_eyre::eyre::eyre!(
                    "Group '{}' has no steps. Each group must have at least one step.",
                    group.id
                ));
            }

            // 检查步骤是否有字段
            for (step_idx, step) in group.steps.iter().enumerate() {
                if step.fields.is_empty() {
                    return Err(color_eyre::eyre::eyre!(
                        "Group '{}', step {} has no fields. Each step must have at least one field.",
                        group.id,
                        step_idx + 1
                    ));
                }
            }
        }

        Ok(())
    }

    /// 执行表单并收集用户输入
    ///
    /// # 返回
    ///
    /// 返回 `FormResult`，包含所有字段的值
    ///
    /// # 错误
    ///
    /// 如果用户取消或验证失败，返回错误
    pub fn run(self) -> Result<FormResult> {
        // 先验证配置
        self.validate()?;

        let mut field_values: HashMap<String, String> = HashMap::new();

        // 按顺序执行每个组
        for group in &self.groups {
            // 如果是可选组，先询问是否配置
            if group.optional {
                let should_configure = if let Some(title) = &group.title {
                    log_break!();
                    log_message!("{}", title);
                    log_break!('-', 40);
                    if let Some(description) = &group.description {
                        log_debug!("{}", description);
                        log_break!();
                    }
                    ConfirmDialog::new(format!("Configure {}?", title))
                        .with_default(group.default_enabled)
                        .prompt()?
                } else {
                    group.default_enabled
                };

                if !should_configure {
                    continue;
                }
            } else {
                // 必填组：显示标题和描述（如果有）
                if let Some(title) = &group.title {
                    log_break!();
                    log_message!("{}", title);
                    log_break!('-', 40);
                }
                if let Some(description) = &group.description {
                    log_debug!("{}", description);
                    log_break!();
                }
            }

            // 执行组内的步骤
            for step in &group.steps {
                if self.should_execute_step(step, &field_values) {
                    for field in &step.fields {
                        if self.should_ask_field(field, &field_values) {
                            self.ask_field(field, &mut field_values)?;
                        }
                    }
                }
            }
        }

        Ok(FormResult {
            values: field_values,
        })
    }

    /// 判断步骤是否应该执行
    fn should_execute_step(&self, step: &FormStep, field_values: &HashMap<String, String>) -> bool {
        match &step.step_type {
            StepType::Unconditional => true,
            StepType::Conditional(condition) => {
                ConditionEvaluator::evaluate(condition, field_values)
            }
            StepType::ConditionalAll(conditions) => {
                conditions.iter().all(|c| ConditionEvaluator::evaluate(c, field_values))
            }
            StepType::ConditionalAny(conditions) => {
                conditions.iter().any(|c| ConditionEvaluator::evaluate(c, field_values))
            }
            StepType::DynamicCondition(f) => {
                let result = FormResult {
                    values: field_values.clone(),
                };
                f(&result)
            }
        }
    }

    /// 判断是否应该询问字段（基于字段的条件）
    fn should_ask_field(&self, field: &FormField, field_values: &HashMap<String, String>) -> bool {
        if let Some(ref condition) = field.condition {
            ConditionEvaluator::evaluate(condition, field_values)
        } else {
            true
        }
    }

    /// 询问单个字段
    fn ask_field(
        &self,
        field: &FormField,
        field_values: &mut HashMap<String, String>,
    ) -> Result<()> {
        match field.field_type {
            FormFieldType::Text => {
                let mut dialog = InputDialog::new(&field.message);

                // 设置默认值
                if let Some(ref default_value) = field.default_value {
                    if let Some(default_str) = <FieldDefaultValue>::as_string(default_value) {
                        dialog = dialog.with_default(default_str);
                    }
                }

                // 设置验证器和空值处理
                let field_name = field.name.clone();
                let field_required = field.required;
                let field_allow_empty = field.allow_empty;

                if let Some(ref validator) = field.validator {
                    let validator_clone = validator.clone();
                    dialog = dialog.with_validator(move |input: &str| {
                        // 如果必填且为空，返回错误
                        if field_required && input.trim().is_empty() {
                            return Err(format!("Field '{}' is required", field_name));
                        }
                        // 调用自定义验证器
                        validator_clone(input)
                    });
                } else if field.required {
                    // 如果没有验证器但必填，添加默认验证
                    let field_name = field.name.clone();
                    dialog = dialog.with_validator(move |input: &str| {
                        if input.trim().is_empty() {
                            Err(format!("Field '{}' is required", field_name))
                        } else {
                            Ok(())
                        }
                    });
                } else {
                    // 根据 allow_empty 设置
                    dialog = dialog.allow_empty(field_allow_empty);
                }

                let value = dialog.prompt()?;
                field_values.insert(field.name.clone(), value);
            }
            FormFieldType::Password => {
                let mut password_prompt = Password::new().with_prompt(&field.message);

                // 如果允许空值，设置允许空密码
                if field.allow_empty {
                    password_prompt = password_prompt.allow_empty_password(true);
                }

                let password = password_prompt
                    .interact()
                    .map_err(|e| eyre!("Failed to get password: {}", e))?;

                // 验证必填和空值
                if field.required && password.is_empty() {
                    color_eyre::eyre::bail!("Field '{}' is required", field.name);
                }

                // 如果允许空值且为空，跳过验证器
                if field.allow_empty && password.is_empty() {
                    // 允许空值，直接插入空字符串
                } else {
                    // 验证器
                    if let Some(ref validator) = field.validator {
                        validator(&password)
                            .map_err(|e| color_eyre::eyre::eyre!("Validation error: {}", e))?;
                    }
                }

                field_values.insert(field.name.clone(), password);
            }
            FormFieldType::Selection => {
                let mut dialog = SelectDialog::new(&field.message, field.choices.clone());

                // 设置默认选项
                if let Some(ref default_choice) = field.default_choice {
                    if let Some(idx) = field.choices.iter().position(|c| c == default_choice) {
                        dialog = dialog.with_default(idx);
                    }
                } else if let Some(ref default_value) = field.default_value {
                    if let Some(default_str) = <FieldDefaultValue>::as_string(default_value) {
                        if let Some(idx) = field.choices.iter().position(|c| c == &default_str) {
                            dialog = dialog.with_default(idx);
                        }
                    }
                }

                let value = dialog.prompt()?;
                field_values.insert(field.name.clone(), value);
            }
            FormFieldType::Confirmation => {
                let mut dialog = ConfirmDialog::new(&field.message);

                // 设置默认值
                if let Some(ref default_value) = field.default_value {
                    if let Some(default_bool) = <FieldDefaultValue>::as_bool(default_value) {
                        dialog = dialog.with_default(default_bool);
                    }
                }

                let confirmed = dialog.prompt()?;
                // 将布尔值转换为字符串（"yes" 或 "no"）
                let value = if confirmed {
                    "yes".to_string()
                } else {
                    "no".to_string()
                };
                field_values.insert(field.name.clone(), value);
            }
        }

        Ok(())
    }
}

impl Default for FormBuilder {
    fn default() -> Self {
        Self::new()
    }
}
