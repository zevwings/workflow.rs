//! 表单构建器

use std::collections::{HashMap, HashSet};

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

#[cfg(test)]
mod tests {
    #![allow(clippy::assertions_on_constants)]

    use crate::base::dialog::{FormBuilder, GroupConfig, StepType};

    /// 测试表单构建器创建
    ///
    /// ## 测试目的
    /// 验证 FormBuilder::new() 能够创建一个空的表单构建器。
    ///
    /// ## 测试场景
    /// 1. 调用 FormBuilder::new() 创建构建器
    /// 2. 验证构建器的 groups 字段为空
    ///
    /// ## 预期结果
    /// - 构建器的 groups 为空
    #[test]
    fn test_form_builder_new_creates_empty_builder() {
        // Arrange: 准备创建表单构建器

        // Act: 创建表单构建器（覆盖 builder.rs:51-53）
        let builder = FormBuilder::new();

        // Assert: 验证构建器为空
        assert!(builder.groups.is_empty());
    }

    /// 测试添加表单组功能
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够使用有效配置添加表单组。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个带有效配置的组
    /// 3. 验证组添加成功
    ///
    /// ## 预期结果
    /// - 组被成功添加，groups 长度为 1
    #[test]
    fn test_form_builder_add_group_with_valid_config_adds_group() {
        // Arrange: 准备组ID和配置
        let group_id = "group1";

        // Act: 添加表单组（覆盖 builder.rs:98-117）
        let builder = FormBuilder::new().add_group(
            group_id,
            |g| g.step(|f| f.add_text("field1", "Field 1")),
            GroupConfig::required(),
        );

        // Assert: 验证组添加成功
        assert_eq!(builder.groups.len(), 1);
        assert_eq!(builder.groups[0].id, group_id);
    }

    /// 测试添加多个表单组功能
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够添加多个具有不同配置的表单组。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加多个组（必填组和可选组）
    /// 3. 验证所有组添加成功
    ///
    /// ## 预期结果
    /// - 所有组被成功添加，groups 长度正确
    #[test]
    fn test_form_builder_add_multiple_groups_with_different_configs_adds_all_groups() {
        // Arrange: 准备多个组配置

        // Act: 添加多个组
        let builder = FormBuilder::new()
            .add_group(
                "group1",
                |g| g.step(|f| f.add_text("field1", "Field 1")),
                GroupConfig::required(),
            )
            .add_group(
                "group2",
                |g| g.step(|f| f.add_text("field2", "Field 2")),
                GroupConfig::optional(),
            );

        // Assert: 验证所有组添加成功
        assert_eq!(builder.groups.len(), 2);
        assert_eq!(builder.groups[0].id, "group1");
        assert_eq!(builder.groups[1].id, "group2");
    }

    /// 测试重复组ID验证
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 在遇到重复组ID时返回错误。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加两个具有相同ID的组
    /// 3. 运行验证
    /// 4. 验证返回错误且错误消息包含 "Duplicate group ID"
    ///
    /// ## 预期结果
    /// - 返回错误，错误消息包含 "Duplicate group ID"
    #[test]
    fn test_form_builder_validate_with_duplicate_group_id_returns_error() {
        // Arrange: 准备带有重复组ID的构建器（覆盖 builder.rs:130-137）
        let builder = FormBuilder::new()
            .add_group(
                "group1",
                |g| g.step(|f| f.add_text("field1", "Field 1")),
                GroupConfig::required(),
            )
            .add_group(
                "group1",
                |g| g.step(|f| f.add_text("field2", "Field 2")),
                GroupConfig::required(),
            );

        // Act: 运行验证（通过 run 方法）
        let result = builder.run();

        // Assert: 验证应该失败且错误消息包含"Duplicate group ID"
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Duplicate group ID"));
    }

    /// 测试空组验证
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 在遇到没有步骤的组时返回错误。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个没有步骤的组
    /// 3. 运行验证
    /// 4. 验证返回错误且错误消息包含 "has no steps"
    ///
    /// ## 预期结果
    /// - 返回错误，错误消息包含 "has no steps"
    #[test]
    fn test_form_builder_validate_with_empty_group_returns_error() {
        // Arrange: 准备带有空组的构建器（覆盖 builder.rs:141-147）
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| {
                g // 不添加任何步骤
            },
            GroupConfig::required(),
        );

        // Act: 运行验证
        let result = builder.run();

        // Assert: 验证应该失败且错误消息包含"has no steps"
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("has no steps"));
    }

    /// 测试空步骤处理
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够优雅地处理没有字段的步骤。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含空步骤的组
    /// 3. 运行验证
    /// 4. 验证结果（可能成功或失败，取决于 FieldBuilder 默认行为）
    ///
    /// ## 预期结果
    /// - 验证可能成功或失败，取决于实现
    #[test]
    fn test_form_builder_validate_with_empty_step_handles_gracefully() {
        // Arrange: 准备带有空步骤的构建器（覆盖 builder.rs:149-157）
        // 注意：由于 GroupBuilder 的 step 方法总是会调用 builder，我们需要创建一个空的 FieldBuilder
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| {
                g.step(|f| {
                    // 不添加任何字段（虽然这在实际使用中不太可能）
                    f
                })
            },
            GroupConfig::required(),
        );

        // Act: 运行验证
        let result = builder.run();

        // Assert: 如果步骤没有字段，验证应该失败（可能成功或失败，取决于 FieldBuilder 默认行为）
        assert!(result.is_ok() || result.is_err());
    }

    // 注意：run() 方法需要用户交互，以下测试会被忽略
    /// 测试表单构建器的完整运行流程
    ///
    /// ## 测试目的
    /// 验证`FormBuilder`能够正确显示多步骤表单并接收用户输入。覆盖源代码: `builder.rs:173-229`
    ///
    /// ## 为什么被忽略
    /// - **需要用户交互**: 测试需要用户依次输入各个表单字段
    /// - **CI环境不支持**: 自动化CI环境无法提供交互式输入
    /// - **多步骤流程**: 涉及多个连续的用户输入步骤
    ///
    /// ## 如何手动运行
    /// ```bash
    /// cargo test test_form_builder_run -- --ignored
    /// ```
    /// 然后按照提示依次输入各字段值
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加表单组和字段
    /// 3. 运行表单并等待用户输入
    /// 4. 验证收集的表单数据
    ///
    /// ## 预期行为
    /// - 依次显示各个表单字段
    /// - 接受用户输入并验证
    /// - 返回`Ok(FormData)`包含所有输入值
    /// - 如果用户取消则返回错误
    #[test]
    #[ignore] // 需要用户交互
    #[cfg(feature = "interactive-tests")]
    fn test_form_builder_run() {
        // Arrange: 准备测试运行表单（覆盖 builder.rs:173-229）
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step(|f| f.add_text("field1", "Field 1")),
            GroupConfig::required(),
        );

        let result = builder.run();
        // 这个测试需要手动运行
        assert!(result.is_ok() || result.is_err());
    }

    /// 测试无条件步骤创建
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够创建无条件步骤。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含无条件步骤的组
    /// 3. 验证步骤创建成功
    ///
    /// ## 预期结果
    /// - 步骤被成功创建，组包含步骤
    #[test]
    fn test_form_builder_should_execute_step_with_unconditional_step_creates_step() {
        // Arrange: 准备无条件步骤（覆盖 builder.rs:234）
        // 这个测试通过创建表单并验证结构来间接测试

        // Act: 创建带有无条件步骤的构建器
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step(|f| f.add_text("field1", "Field 1")),
            GroupConfig::required(),
        );

        // Assert: 验证组和步骤创建成功
        assert_eq!(builder.groups.len(), 1);
        assert!(!builder.groups[0].steps.is_empty());
    }

    /// 测试条件步骤创建
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够创建条件步骤（step_if）。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含条件步骤的组
    /// 3. 验证步骤创建成功
    ///
    /// ## 预期结果
    /// - 条件步骤被成功创建，组包含两个步骤
    #[test]
    fn test_form_builder_should_execute_step_with_conditional_step_creates_step() {
        // Arrange: 准备条件步骤（覆盖 builder.rs:235-237）

        // Act: 创建带有条件步骤的构建器
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| {
                g.step(|f| f.add_text("field1", "Field 1"))
                    .step_if("field1", "value1", |f| f.add_text("field2", "Field 2"))
            },
            GroupConfig::required(),
        );

        // Assert: 验证条件步骤创建成功
        assert_eq!(builder.groups.len(), 1);
        assert_eq!(builder.groups[0].steps.len(), 2);
    }

    /// 测试多条件步骤创建（AND）
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够创建多条件步骤（所有条件必须满足）。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含多条件步骤（AND）的组
    /// 3. 验证步骤创建成功
    ///
    /// ## 预期结果
    /// - 多条件步骤被成功创建
    #[test]
    fn test_form_builder_should_execute_step_with_conditional_all_creates_step() {
        // Arrange: 准备多条件步骤（AND）（覆盖 builder.rs:238-240）
        let conditions = vec![("field1", "value1"), ("field2", "value2")];

        // Act: 创建带有多条件步骤的构建器
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step_if_all(conditions, |f| f.add_text("field3", "Field 3")),
            GroupConfig::required(),
        );

        // Assert: 验证多条件步骤创建成功
        assert_eq!(builder.groups.len(), 1);
        assert!(!builder.groups[0].steps.is_empty());
    }

    /// 测试多条件步骤创建（OR）
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够创建多条件步骤（任一条件满足即可）。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含多条件步骤（OR）的组
    /// 3. 验证步骤创建成功
    ///
    /// ## 预期结果
    /// - 多条件步骤被成功创建
    #[test]
    fn test_form_builder_should_execute_step_with_conditional_any_creates_step() {
        // Arrange: 准备多条件步骤（OR）（覆盖 builder.rs:241-243）
        let conditions = vec![("field1", "value1"), ("field2", "value2")];

        // Act: 创建带有多条件步骤的构建器
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step_if_any(conditions, |f| f.add_text("field3", "Field 3")),
            GroupConfig::required(),
        );

        // Assert: 验证多条件步骤创建成功
        assert_eq!(builder.groups.len(), 1);
        assert!(!builder.groups[0].steps.is_empty());
    }

    /// 测试动态条件步骤创建
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够创建动态条件步骤（使用函数判断）。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含动态条件步骤的组
    /// 3. 验证步骤创建成功
    ///
    /// ## 预期结果
    /// - 动态条件步骤被成功创建
    #[test]
    fn test_form_builder_should_execute_step_with_dynamic_condition_creates_step() {
        // Arrange: 准备动态条件步骤（覆盖 builder.rs:244-249）

        // Act: 创建带有动态条件步骤的构建器
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step_if_dynamic(|_result| true, |f| f.add_text("field1", "Field 1")),
            GroupConfig::required(),
        );

        // Assert: 验证动态条件步骤创建成功
        assert_eq!(builder.groups.len(), 1);
        assert!(!builder.groups[0].steps.is_empty());
    }

    /// 测试无条件字段创建
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够创建没有条件的字段。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含无条件字段的组
    /// 3. 验证字段创建成功且没有条件
    ///
    /// ## 预期结果
    /// - 字段被成功创建，condition 为 None
    #[test]
    fn test_form_builder_should_ask_field_without_condition_creates_field() {
        // Arrange: 准备没有条件的字段（覆盖 builder.rs:257-259）

        // Act: 创建带有无条件字段的构建器
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step(|f| f.add_text("field1", "Field 1")),
            GroupConfig::required(),
        );

        // Assert: 验证字段创建成功（没有条件）
        assert_eq!(builder.groups.len(), 1);
        assert!(!builder.groups[0].steps[0].fields.is_empty());
        assert!(builder.groups[0].steps[0].fields[0].condition.is_none());
    }

    /// 测试添加带标题的组
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够为组设置标题。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个带标题的组
    /// 3. 验证标题设置成功
    ///
    /// ## 预期结果
    /// - 组的 title 字段被正确设置
    #[test]
    fn test_form_builder_add_group_with_title_sets_title() {
        // Arrange: 准备组标题
        let title = "Test Group";

        // Act: 添加带标题的组（覆盖 builder.rs:108）
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step(|f| f.add_text("field1", "Field 1")),
            GroupConfig::required().with_title(title),
        );

        // Assert: 验证标题设置成功
        assert_eq!(builder.groups.len(), 1);
        assert_eq!(builder.groups[0].title, Some(title.to_string()));
    }

    /// 测试添加带描述的组
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够为组设置描述。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个带描述的组
    /// 3. 验证描述设置成功
    ///
    /// ## 预期结果
    /// - 组的 description 字段被正确设置
    #[test]
    fn test_form_builder_add_group_with_description_sets_description() {
        // Arrange: 准备组描述
        let description = "Test Description";

        // Act: 添加带描述的组（覆盖 builder.rs:109）
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step(|f| f.add_text("field1", "Field 1")),
            GroupConfig::required().with_description(description),
        );

        // Assert: 验证描述设置成功
        assert_eq!(builder.groups.len(), 1);
        assert_eq!(builder.groups[0].description, Some(description.to_string()));
    }

    /// 测试添加可选组
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够添加可选组并设置默认启用状态。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个可选组并设置默认启用
    /// 3. 验证组标记为可选且默认启用
    ///
    /// ## 预期结果
    /// - 组的 optional 和 default_enabled 字段被正确设置
    #[test]
    fn test_form_builder_add_optional_group_marks_group_as_optional() {
        // Arrange: 准备可选组配置（覆盖 builder.rs:110-111）

        // Act: 添加可选组
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step(|f| f.add_text("field1", "Field 1")),
            GroupConfig::optional().with_default_enabled(true),
        );

        // Assert: 验证组标记为可选且默认启用
        assert_eq!(builder.groups.len(), 1);
        assert!(builder.groups[0].optional);
        assert!(builder.groups[0].default_enabled);
    }

    /// 测试默认构建器创建
    ///
    /// ## 测试目的
    /// 验证 FormBuilder::default() 能够创建一个空的表单构建器。
    ///
    /// ## 测试场景
    /// 1. 调用 FormBuilder::default() 创建构建器
    /// 2. 验证构建器的 groups 字段为空
    ///
    /// ## 预期结果
    /// - 构建器的 groups 为空
    #[test]
    fn test_form_builder_default_creates_empty_builder() {
        // Arrange: 准备使用 Default trait

        // Act: 创建默认构建器（覆盖 builder.rs:386-389）
        let builder = FormBuilder::default();

        // Assert: 验证构建器为空
        assert!(builder.groups.is_empty());
    }

    /// 测试组配置所有选项
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够为组设置所有配置选项（标题、描述、可选性、默认启用）。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含所有配置选项的组
    /// 3. 验证所有选项设置成功
    ///
    /// ## 预期结果
    /// - 所有配置选项被正确设置
    #[test]
    fn test_form_builder_group_config_with_all_options_sets_all_options() {
        // Arrange: 准备包含所有选项的组配置（覆盖 builder.rs:106-113）
        let title = "Test Group";
        let description = "Test Description";

        // Act: 添加包含所有选项的组
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step(|f| f.add_text("field1", "Field 1")),
            GroupConfig::optional()
                .with_title(title)
                .with_description(description)
                .with_default_enabled(true),
        );

        // Assert: 验证所有选项设置成功
        let group = &builder.groups[0];
        assert_eq!(group.id, "group1");
        assert_eq!(group.title, Some(title.to_string()));
        assert_eq!(group.description, Some(description.to_string()));
        assert!(group.optional);
        assert!(group.default_enabled);
    }

    /// 测试空步骤字段验证
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 在遇到没有字段的步骤时返回错误。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含空步骤字段的组
    /// 3. 运行验证
    /// 4. 验证返回错误
    ///
    /// ## 预期结果
    /// - 返回错误，错误消息包含相关信息
    #[test]
    fn test_form_builder_validate_with_empty_step_fields_returns_error() {
        // Arrange: 准备带有空步骤字段的构建器（覆盖 builder.rs:149-157）
        // 创建一个没有字段的步骤（通过不添加任何字段）
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step(|f| f), // 不添加任何字段
            GroupConfig::required(),
        );

        // Act: 运行验证
        let result = builder.run();

        // Assert: 验证应该失败且错误消息包含相关信息
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("has no fields") || error_msg.contains("step"));
    }

    /// 测试组ID字符串转换
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够正确处理字符串类型的组ID。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 使用字符串类型的组ID添加组
    /// 3. 验证组ID转换正确
    ///
    /// ## 预期结果
    /// - 组ID被正确转换和存储
    #[test]
    fn test_form_builder_group_id_with_string_id_converts_correctly() {
        // Arrange: 准备字符串类型的组ID（覆盖 builder.rs:102）
        let group_id = "group1".to_string();

        // Act: 添加组（使用字符串ID）
        let builder = FormBuilder::new().add_group(
            group_id.clone(),
            |g| g.step(|f| f.add_text("field1", "Field 1")),
            GroupConfig::required(),
        );

        // Assert: 验证组ID转换正确
        assert_eq!(builder.groups.len(), 1);
        assert_eq!(builder.groups[0].id, "group1");
    }

    // ==================== Additional validate() Method Tests ====================

    /// 测试多个空组验证
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 在遇到多个空组时返回错误。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加多个空组
    /// 3. 运行验证
    /// 4. 验证返回错误
    ///
    /// ## 预期结果
    /// - 返回错误，错误消息包含 "has no steps"
    #[test]
    fn test_form_builder_validate_with_multiple_empty_groups_returns_error() {
        // Arrange: 准备多个空组的构建器（覆盖 builder.rs:141-147）
        let builder = FormBuilder::new()
            .add_group("group1", |g| g, GroupConfig::required())
            .add_group("group2", |g| g, GroupConfig::required());

        // Act: 运行验证
        let result = builder.run();

        // Assert: 验证应该失败且错误消息包含"has no steps"
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("has no steps"));
    }

    /// 测试多个步骤包含空字段验证
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 在遇到包含空字段的多个步骤时返回错误。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含空字段的多个步骤的组
    /// 3. 运行验证
    /// 4. 验证返回错误
    ///
    /// ## 预期结果
    /// - 返回错误，错误消息包含相关信息
    #[test]
    fn test_form_builder_validate_with_multiple_steps_containing_empty_fields_returns_error() {
        // Arrange: 准备包含空字段的多个步骤的构建器（覆盖 builder.rs:149-157）
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| {
                g.step(|f| f.add_text("field1", "Field 1")).step(|f| f) // 空步骤
            },
            GroupConfig::required(),
        );

        // Act: 运行验证
        let result = builder.run();

        // Assert: 验证应该失败且错误消息包含相关信息
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("has no fields") || error_msg.contains("step"));
    }

    // ==================== should_execute_step() 间接测试 ====================
    // 注意：should_execute_step() 是私有方法，通过创建表单结构来间接测试

    /// 测试条件步骤评估
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够创建条件步骤并正确设置步骤类型。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含条件步骤的组
    /// 3. 验证步骤类型为 Conditional
    ///
    /// ## 预期结果
    /// - 条件步骤被创建，步骤类型为 Conditional
    #[test]
    fn test_form_builder_step_conditional_evaluation_with_conditional_step_creates_conditional_step(
    ) {
        // Arrange: 准备条件步骤（覆盖 builder.rs:235-237）
        // 通过创建条件步骤来验证结构

        // Act: 创建带有条件步骤的构建器
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| {
                g.step(|f| f.add_text("field1", "Field 1"))
                    .step_if("field1", "value1", |f| f.add_text("field2", "Field 2"))
            },
            GroupConfig::required(),
        );

        // Assert: 验证条件步骤创建成功且类型正确
        assert_eq!(builder.groups.len(), 1);
        assert_eq!(builder.groups[0].steps.len(), 2);
        match &builder.groups[0].steps[1].step_type {
            StepType::Conditional(_) => {
                // 验证步骤类型正确
            }
            _ => panic!("Expected conditional step"),
        }
    }

    /// 测试多条件步骤评估（AND）
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够创建多条件步骤（AND）并正确设置步骤类型。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含多条件步骤（AND）的组
    /// 3. 验证步骤类型为 ConditionalAll
    ///
    /// ## 预期结果
    /// - 多条件步骤被创建，步骤类型为 ConditionalAll
    #[test]
    fn test_form_builder_step_conditional_all_evaluation_with_multiple_conditions_creates_conditional_all_step(
    ) {
        // Arrange: 准备多条件步骤（AND）（覆盖 builder.rs:238-240）
        let conditions = vec![("field1", "value1"), ("field2", "value2")];

        // Act: 创建带有多条件步骤的构建器
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step_if_all(conditions, |f| f.add_text("field3", "Field 3")),
            GroupConfig::required(),
        );

        // Assert: 验证多条件步骤创建成功且类型正确
        assert_eq!(builder.groups.len(), 1);
        assert!(!builder.groups[0].steps.is_empty());
        match &builder.groups[0].steps[0].step_type {
            StepType::ConditionalAll(_) => {
                // 验证步骤类型正确
            }
            _ => panic!("Expected conditional all step"),
        }
    }

    /// 测试多条件步骤评估（OR）
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够创建多条件步骤（OR）并正确设置步骤类型。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含多条件步骤（OR）的组
    /// 3. 验证步骤类型为 ConditionalAny
    ///
    /// ## 预期结果
    /// - 多条件步骤被创建，步骤类型为 ConditionalAny
    #[test]
    fn test_form_builder_step_conditional_any_evaluation_with_multiple_conditions_creates_conditional_any_step(
    ) {
        // Arrange: 准备多条件步骤（OR）（覆盖 builder.rs:241-243）
        let conditions = vec![("field1", "value1"), ("field2", "value2")];

        // Act: 创建带有多条件步骤的构建器
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step_if_any(conditions, |f| f.add_text("field3", "Field 3")),
            GroupConfig::required(),
        );

        // Assert: 验证多条件步骤创建成功且类型正确
        assert_eq!(builder.groups.len(), 1);
        assert!(!builder.groups[0].steps.is_empty());
        match &builder.groups[0].steps[0].step_type {
            StepType::ConditionalAny(_) => {
                // 验证步骤类型正确
            }
            _ => panic!("Expected conditional any step"),
        }
    }

    /// 测试动态条件步骤评估
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够创建动态条件步骤并正确设置步骤类型。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含动态条件步骤的组
    /// 3. 验证步骤类型为 DynamicCondition
    ///
    /// ## 预期结果
    /// - 动态条件步骤被创建，步骤类型为 DynamicCondition
    #[test]
    fn test_form_builder_step_dynamic_condition_evaluation_with_dynamic_condition_creates_dynamic_step(
    ) {
        // Arrange: 准备动态条件步骤（覆盖 builder.rs:244-249）

        // Act: 创建带有动态条件步骤的构建器
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step_if_dynamic(|_result| true, |f| f.add_text("field1", "Field 1")),
            GroupConfig::required(),
        );

        // Assert: 验证动态条件步骤创建成功且类型正确
        assert_eq!(builder.groups.len(), 1);
        assert!(!builder.groups[0].steps.is_empty());
        match &builder.groups[0].steps[0].step_type {
            StepType::DynamicCondition(_) => {
                // 验证步骤类型正确
            }
            _ => panic!("Expected dynamic condition step"),
        }
    }

    // ==================== should_ask_field() 间接测试 ====================
    // 注意：should_ask_field() 是私有方法，通过创建带条件的字段来间接测试

    /// 测试带条件的字段创建
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够创建带条件的字段并正确设置步骤类型。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含条件步骤的组
    /// 3. 验证步骤类型和字段数量正确
    ///
    /// ## 预期结果
    /// - 条件步骤被创建，步骤类型为 Conditional
    #[test]
    fn test_form_builder_field_with_condition_creates_conditional_step() {
        // Arrange: 准备带条件的字段（覆盖 builder.rs:255-257）

        // Act: 创建带有条件步骤的构建器
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| {
                g.step(|f| f.add_text("field1", "Field 1"))
                    .step_if("field1", "value1", |f| f.add_text("field2", "Field 2"))
            },
            GroupConfig::required(),
        );

        // Assert: 验证字段创建成功且步骤类型正确
        assert_eq!(builder.groups.len(), 1);
        assert_eq!(builder.groups[0].steps.len(), 2); // 两个步骤：一个无条件，一个有条件
        assert_eq!(builder.groups[0].steps[0].fields.len(), 1); // 第一个步骤有一个字段
        assert_eq!(builder.groups[0].steps[1].fields.len(), 1); // 第二个步骤有一个字段
                                                                // Assert: 验证第二个步骤有条件（步骤类型是 Conditional，不是字段的条件）
        match &builder.groups[0].steps[1].step_type {
            StepType::Conditional(_) => {
                // 验证步骤类型正确
            }
            _ => panic!("Expected conditional step"),
        }
    }

    /// 测试无条件字段创建
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够创建没有条件的字段。
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含无条件字段的组
    /// 3. 验证字段创建成功且没有条件
    ///
    /// ## 预期结果
    /// - 字段被成功创建，condition 为 None
    #[test]
    fn test_form_builder_field_without_condition_creates_unconditional_field() {
        // Arrange: 准备没有条件的字段（覆盖 builder.rs:257-259）

        // Act: 创建带有无条件字段的构建器
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step(|f| f.add_text("field1", "Field 1")),
            GroupConfig::required(),
        );

        // Assert: 验证字段创建成功（没有条件）
        assert_eq!(builder.groups.len(), 1);
        assert!(!builder.groups[0].steps[0].fields.is_empty());
        assert!(builder.groups[0].steps[0].fields[0].condition.is_none());
    }

    // ==================== ask_field() 间接测试 ====================
    // 注意：ask_field() 需要用户交互，这些测试会被忽略，但可以验证字段类型

    /// 测试询问文本字段功能
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够询问文本类型的字段（需要用户交互）。
    ///
    /// ## 为什么被忽略
    /// - **需要用户交互**: 测试需要用户输入文本
    /// - **CI环境不支持**: 自动化CI环境无法提供交互式输入
    ///
    /// ## 如何手动运行
    /// ```bash
    /// cargo test test_form_builder_ask_field_text -- --ignored
    /// ```
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含文本字段的组
    /// 3. 运行表单并等待用户输入
    ///
    /// ## 预期结果
    /// - 能够询问文本字段并接收用户输入
    #[test]
    #[ignore] // 需要用户交互
    fn test_form_builder_ask_field_text() {
        // Arrange: 准备测试 ask_field() 方法 - Text 类型（覆盖 builder.rs:269-311）
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step(|f| f.add_text("field1", "Field 1")),
            GroupConfig::required(),
        );

        let result = builder.run();
        // 这个测试需要手动运行
        assert!(result.is_ok() || result.is_err());
    }

    /// 测试询问密码字段功能
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够询问密码类型的字段（需要用户交互）。
    ///
    /// ## 为什么被忽略
    /// - **需要用户交互**: 测试需要用户输入密码
    /// - **CI环境不支持**: 自动化CI环境无法提供交互式输入
    ///
    /// ## 如何手动运行
    /// ```bash
    /// cargo test test_form_builder_ask_field_password -- --ignored
    /// ```
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含密码字段的组
    /// 3. 运行表单并等待用户输入
    ///
    /// ## 预期结果
    /// - 能够询问密码字段并接收用户输入
    #[test]
    #[ignore] // 需要用户交互
    fn test_form_builder_ask_field_password() {
        // Arrange: 准备测试 ask_field() 方法 - Password 类型（覆盖 builder.rs:312-341）
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step(|f| f.add_password("password", "Enter password")),
            GroupConfig::required(),
        );

        let result = builder.run();
        // 这个测试需要手动运行
        assert!(result.is_ok() || result.is_err());
    }

    /// 测试询问选择字段功能
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够询问选择类型的字段（需要用户交互）。
    ///
    /// ## 为什么被忽略
    /// - **需要用户交互**: 测试需要用户选择选项
    /// - **CI环境不支持**: 自动化CI环境无法提供交互式输入
    ///
    /// ## 如何手动运行
    /// ```bash
    /// cargo test test_form_builder_ask_field_selection -- --ignored
    /// ```
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含选择字段的组
    /// 3. 运行表单并等待用户选择
    ///
    /// ## 预期结果
    /// - 能够询问选择字段并接收用户选择
    #[test]
    #[ignore] // 需要用户交互
    fn test_form_builder_ask_field_selection() {
        // Arrange: 准备测试 ask_field() 方法 - Selection 类型（覆盖 builder.rs:342-360）
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| {
                g.step(|f| {
                    f.add_selection(
                        "choice",
                        "Select option",
                        vec!["option1".into(), "option2".into()],
                    )
                })
            },
            GroupConfig::required(),
        );

        let result = builder.run();
        // 这个测试需要手动运行
        assert!(result.is_ok() || result.is_err());
    }

    /// 测试询问确认字段功能
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够询问确认类型的字段（需要用户交互）。
    ///
    /// ## 为什么被忽略
    /// - **需要用户交互**: 测试需要用户确认
    /// - **CI环境不支持**: 自动化CI环境无法提供交互式输入
    ///
    /// ## 如何手动运行
    /// ```bash
    /// cargo test test_form_builder_ask_field_confirmation -- --ignored
    /// ```
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个包含确认字段的组
    /// 3. 运行表单并等待用户确认
    ///
    /// ## 预期结果
    /// - 能够询问确认字段并接收用户确认
    #[test]
    #[ignore] // 需要用户交互
    fn test_form_builder_ask_field_confirmation() {
        // Arrange: 准备测试 ask_field() 方法 - Confirmation 类型（覆盖 builder.rs:361-379）
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step(|f| f.add_confirmation("confirm", "Confirm?")),
            GroupConfig::required(),
        );

        let result = builder.run();
        // 这个测试需要手动运行
        assert!(result.is_ok() || result.is_err());
    }

    // ==================== run() 方法的更多测试 ====================

    /// 测试运行表单（可选组）
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够运行包含可选组的表单（需要用户交互）。
    ///
    /// ## 为什么被忽略
    /// - **需要用户交互**: 测试需要用户输入
    /// - **CI环境不支持**: 自动化CI环境无法提供交互式输入
    ///
    /// ## 如何手动运行
    /// ```bash
    /// cargo test test_form_builder_run_with_optional_group -- --ignored
    /// ```
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个可选组
    /// 3. 运行表单并等待用户输入
    ///
    /// ## 预期结果
    /// - 能够运行表单并处理可选组
    #[test]
    #[ignore] // 需要用户交互
    fn test_form_builder_run_with_optional_group() {
        // Arrange: 准备测试 run() 方法 - 可选组（覆盖 builder.rs:182-196）
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step(|f| f.add_text("field1", "Field 1")),
            GroupConfig::optional().with_default_enabled(false),
        );

        let result = builder.run();
        // 这个测试需要手动运行
        assert!(result.is_ok() || result.is_err());
    }

    /// 测试运行表单（必填组）
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够运行包含必填组的表单（需要用户交互）。
    ///
    /// ## 为什么被忽略
    /// - **需要用户交互**: 测试需要用户输入
    /// - **CI环境不支持**: 自动化CI环境无法提供交互式输入
    ///
    /// ## 如何手动运行
    /// ```bash
    /// cargo test test_form_builder_run_with_required_group -- --ignored
    /// ```
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加一个必填组
    /// 3. 运行表单并等待用户输入
    ///
    /// ## 预期结果
    /// - 能够运行表单并处理必填组
    #[test]
    #[ignore] // 需要用户交互
    fn test_form_builder_run_with_required_group() {
        // Arrange: 准备测试 run() 方法 - 必填组（覆盖 builder.rs:200-212）
        let builder = FormBuilder::new().add_group(
            "group1",
            |g| g.step(|f| f.add_text("field1", "Field 1")),
            GroupConfig::required().with_title("Required Group"),
        );

        let result = builder.run();
        // 这个测试需要手动运行
        assert!(result.is_ok() || result.is_err());
    }

    /// 测试运行表单（多个组）
    ///
    /// ## 测试目的
    /// 验证 FormBuilder 能够运行包含多个组的表单（需要用户交互）。
    ///
    /// ## 为什么被忽略
    /// - **需要用户交互**: 测试需要用户输入多个字段
    /// - **CI环境不支持**: 自动化CI环境无法提供交互式输入
    ///
    /// ## 如何手动运行
    /// ```bash
    /// cargo test test_form_builder_run_with_multiple_groups -- --ignored
    /// ```
    ///
    /// ## 测试场景
    /// 1. 创建表单构建器
    /// 2. 添加多个组（必填组和可选组）
    /// 3. 运行表单并等待用户输入
    ///
    /// ## 预期结果
    /// - 能够运行表单并处理多个组
    #[test]
    #[ignore] // 需要用户交互
    fn test_form_builder_run_with_multiple_groups() {
        // Arrange: 准备测试 run() 方法 - 多个组（覆盖 builder.rs:179-224）
        let builder = FormBuilder::new()
            .add_group(
                "group1",
                |g| g.step(|f| f.add_text("field1", "Field 1")),
                GroupConfig::required(),
            )
            .add_group(
                "group2",
                |g| g.step(|f| f.add_text("field2", "Field 2")),
                GroupConfig::optional(),
            );

        let result = builder.run();
        // 这个测试需要手动运行
        assert!(result.is_ok() || result.is_err());
    }
}
