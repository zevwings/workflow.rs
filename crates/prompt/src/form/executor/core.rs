//! 表单执行器核心逻辑

use crate::dialog::Result;
use crate::form::builder::FormBuilder;
use crate::form::executor::field_executors::FieldExecutors;
use crate::form::executor::separator::{print_nested_form_separator_simple, print_separator};
use crate::form::field::{FieldType, FormField};
use crate::form::types::{FormResult, FormStep, StepType};
use crate::PromptError;

/// 表单执行器
pub struct FormExecutor;

impl Default for FormExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl FormExecutor {
    /// 创建新的表单执行器
    pub fn new() -> Self {
        Self
    }

    /// 执行表单字段序列
    pub fn execute(&self, builder: &FormBuilder) -> Result<FormResult> {
        // 检查是否使用 Group 模式
        if builder.has_groups() {
            self.execute_groups(builder)
        } else {
            self.execute_with_level(builder, 0)
        }
    }

    /// 执行 Group 模式
    fn execute_groups(&self, builder: &FormBuilder) -> Result<FormResult> {
        let mut result = FormResult::new();
        let groups = builder.get_groups();

        // 按顺序执行每个组
        for group in groups {
            // 如果是可选组，先询问是否配置
            if group.optional {
                let should_configure = if let Some(title) = &group.title {
                    crate::br!();
                    crate::separator!('─', 80, title);
                    if let Some(description) = &group.description {
                        crate::debug!("{}", description);
                        crate::br!();
                    }
                    crate::confirm!("Configure {}?", title)
                        .default(group.default_enabled)
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
                    crate::br!();
                    crate::separator!('─', 80, title);
                }
                if let Some(description) = &group.description {
                    crate::debug!("{}", description);
                    crate::br!();
                }
            }

            // 执行组内的步骤
            for step in &group.steps {
                if self.should_execute_step(step, &result) {
                    for field in &step.fields {
                        // 评估字段条件（如果有）
                        if let Some(condition) = &field.condition {
                            if !condition(&result) {
                                // 条件不满足，跳过该字段
                                continue;
                            }
                        }

                        // 执行字段
                        // 注意：如果字段执行失败（例如用户取消或输入错误），
                        // 错误会向上传播，导致整个表单执行失败，之前已保存的字段也会丢失
                        let value = self.execute_field(field, &result, 0)
                            .map_err(|e| {
                                // 提供更详细的错误信息，包含当前已收集的字段信息
                                let collected_fields: Vec<String> = result.values.keys().cloned().collect();
                                crate::error::PromptError::Terminal(
                                    format!(
                                        "Failed to execute field '{}': {}. \
                                        Already collected fields: {:?}. \
                                        This may indicate an error during input (e.g., paste operation failed).",
                                        field.key,
                                        e,
                                        collected_fields
                                    )
                                )
                            })?;

                        // 收集结果（使用 set_boxed 因为 value 已经是 Box<dyn std::any::Any + Send + Sync>）
                        result.set_boxed(field.key.clone(), value);
                    }
                }
            }
        }

        Ok(result)
    }

    /// 判断步骤是否应该执行
    fn should_execute_step(&self, step: &FormStep, result: &FormResult) -> bool {
        match &step.step_type {
            StepType::Unconditional => true,
            StepType::Conditional(condition) => condition(result),
            StepType::ConditionalAll(conditions) => conditions.iter().all(|c| c(result)),
            StepType::ConditionalAny(conditions) => conditions.iter().any(|c| c(result)),
            StepType::DynamicCondition(f) => f(result),
        }
    }

    /// 执行单个字段
    fn execute_field(
        &self,
        field: &FormField,
        _current_result: &FormResult,
        level: usize,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let executors = FieldExecutors;
        match field.field_type {
            FieldType::Confirm => executors.execute_confirm_field(field),
            FieldType::Input => executors.execute_input_field(field, false),
            FieldType::Password => executors.execute_input_field(field, true),
            FieldType::Select => executors.execute_select_field(field),
            FieldType::MultiSelect => executors.execute_multiselect_field(field),
            FieldType::Form => self.execute_nested_form(field, level),
        }
    }

    /// 执行嵌套表单
    pub(super) fn execute_with_level(
        &self,
        builder: &FormBuilder,
        level: usize,
    ) -> Result<FormResult> {
        let title = builder.get_title();

        // 判断是主表单（level == 0）还是嵌套表单（level > 0）
        let is_main_form = level == 0;

        // 输出开始分割线
        if let Some(title) = title {
            if is_main_form {
                // 主表单：显示开始和结束分割线（带 Start/End 后缀）
                print_separator(title, "start", is_main_form)?;
            } else {
                // 嵌套表单：只显示开始分割线（不带 Start/End 后缀）
                print_nested_form_separator_simple(title)?;
            }
        }

        let mut result = FormResult::new();
        let fields = builder.get_fields();

        for field in fields {
            // 评估条件（如果有）
            if let Some(condition) = &field.condition {
                if !condition(&result) {
                    // 条件不满足，跳过该字段
                    continue;
                }
            }

            // 执行字段
            let value = self.execute_field(field, &result, level)?;

            // 收集结果（使用 set_boxed 因为 value 已经是 Box<dyn std::any::Any + Send + Sync>）
            result.set_boxed(field.key.clone(), value);
        }

        // 输出结束分割线（仅主表单显示）
        if let Some(title) = title {
            if is_main_form {
                print_separator(title, "end", is_main_form)?;
            }
        }

        Ok(result)
    }

    /// 执行嵌套表单
    fn execute_nested_form(
        &self,
        field: &FormField,
        level: usize,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let nested_form = field
            .nested_form
            .as_ref()
            .ok_or_else(|| PromptError::InvalidInput("嵌套表单不能为空".to_string()))?;

        let nested_result = self.execute_with_level(nested_form, level + 1)?;
        Ok(Box::new(nested_result))
    }
}
