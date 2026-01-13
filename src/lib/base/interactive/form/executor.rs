//! 表单执行器

use crate::base::interactive::dialog::{PromptError, Result};
use crate::base::interactive::form::builder::FormBuilder;
use crate::base::interactive::form::field::{FieldType, FormField};
use crate::base::interactive::form::group::{FormStep, StepType};
use crate::base::interactive::form::result::FormResult;
use std::io::Write;
use unicode_width::UnicodeWidthStr;

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
                    crate::info!("{}", title);
                    crate::br!('-', 40);
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
                    crate::info!("{}", title);
                    crate::br!('-', 40);
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
                        let value = self.execute_field(field, &result, 0)?;

                        // 收集结果
                        result.set(field.key.clone(), value);
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

    /// 执行表单字段序列（带层级信息，用于简单模式和嵌套表单）
    fn execute_with_level(&self, builder: &FormBuilder, level: usize) -> Result<FormResult> {
        let title = builder.get_title();

        // 判断是主表单（level == 0）还是嵌套表单（level > 0）
        let is_main_form = level == 0;

        // 输出开始分割线
        if let Some(title) = title {
            if is_main_form {
                // 主表单：显示开始和结束分割线（带 Start/End 后缀）
                self.print_separator(title, "start", is_main_form)?;
            } else {
                // 嵌套表单：只显示开始分割线（不带 Start/End 后缀）
                self.print_nested_form_separator_simple(title)?;
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

            // 收集结果
            result.set(field.key.clone(), value);
        }

        // 输出结束分割线（仅主表单显示）
        if let Some(title) = title {
            if is_main_form {
                self.print_separator(title, "end", is_main_form)?;
            }
        }

        Ok(result)
    }

    /// 执行单个字段
    fn execute_field(
        &self,
        field: &FormField,
        _current_result: &FormResult,
        level: usize,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        match field.field_type {
            FieldType::Confirm => self.execute_confirm_field(field),
            FieldType::Input => self.execute_input_field(field, false),
            FieldType::Password => self.execute_input_field(field, true),
            FieldType::Select => self.execute_select_field(field),
            FieldType::MultiSelect => self.execute_multiselect_field(field),
            FieldType::Form => self.execute_nested_form(field, level),
        }
    }

    /// 执行确认字段
    fn execute_confirm_field(
        &self,
        field: &FormField,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let default_value = field
            .default_value
            .as_ref()
            .and_then(|v| v.downcast_ref::<bool>())
            .copied()
            .unwrap_or(false);

        let confirmed = crate::base::interactive::dialog::ConfirmBuilder::new(&field.prompt)
            .default(default_value)
            .prompt()?;

        Ok(Box::new(confirmed))
    }

    /// 执行输入字段（Input 或 Password）
    fn execute_input_field(
        &self,
        field: &FormField,
        is_password: bool,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let default_value = field
            .default_value
            .as_ref()
            .and_then(|v| v.downcast_ref::<String>())
            .cloned()
            .unwrap_or_default();

        let mut builder = if is_password {
            crate::base::interactive::dialog::InputBuilder::new(&field.prompt).password()
        } else {
            crate::base::interactive::dialog::InputBuilder::new(&field.prompt)
        };

        if !default_value.is_empty() {
            builder = builder.default(default_value);
        }

        if let Some(ref result_title) = field.result_title {
            builder = builder.result_title(result_title);
        }

        // 应用验证器（如果存在）
        if let Some(validator) = &field.validator {
            builder = builder.validator(ArcValidatorAdapter(std::sync::Arc::clone(validator)));
        }

        let value = builder.prompt()?;
        Ok(Box::new(value))
    }

    /// 执行选择字段
    fn execute_select_field(
        &self,
        field: &FormField,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let default_index = field.default_index.unwrap_or(0);
        let selected = crate::select!(field.prompt.clone(), field.options.clone())
            .default(default_index)
            .prompt()?;

        // 返回选中的选项值（String），而不是索引，以兼容旧 API
        Ok(Box::new(selected))
    }

    /// 执行多选字段
    fn execute_multiselect_field(
        &self,
        field: &FormField,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let selected = crate::base::interactive::dialog::MultiSelectBuilder::new(
            &field.prompt,
            field.options.clone(),
        )
        .default(field.default_selected.clone())
        .prompt()?;

        // 找到选中项的索引列表
        let indices: Vec<usize> = selected
            .iter()
            .filter_map(|item| {
                field
                    .options
                    .iter()
                    .enumerate()
                    .find(|(_, opt)| *opt == item)
                    .map(|(idx, _)| idx)
            })
            .collect();

        Ok(Box::new(indices))
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

    /// 打印分割线
    fn print_separator(&self, title: &str, suffix: &str, is_main_form: bool) -> Result<()> {
        const SEPARATOR_CHAR: &str = "─";
        const SEPARATOR_LENGTH: usize = 72;

        // 构建文本：title + " " + suffix（首字母大写）
        let suffix_capitalized = if !suffix.is_empty() {
            let mut chars = suffix.chars();
            if let Some(first) = chars.next() {
                format!("{}{}", first.to_uppercase(), chars.as_str())
            } else {
                suffix.to_string()
            }
        } else {
            suffix.to_string()
        };
        let text = format!("{} {}", title, suffix_capitalized);

        self.print_separator_line(&text, SEPARATOR_CHAR, SEPARATOR_LENGTH, is_main_form)
    }

    /// 打印嵌套表单分割线（单行格式，不带 Start/End 后缀）
    fn print_nested_form_separator_simple(&self, title: &str) -> Result<()> {
        const SEPARATOR_CHAR: &str = "─";
        const SEPARATOR_LENGTH: usize = 72;
        self.print_separator_line(title, SEPARATOR_CHAR, SEPARATOR_LENGTH, false)
    }

    /// 打印分割线（统一方法）
    fn print_separator_line(
        &self,
        text: &str,
        separator_char: &str,
        total_width: usize,
        format_main: bool,
    ) -> Result<()> {
        let mut stdout = std::io::stdout();
        writeln!(stdout)?;
        stdout.flush()?;

        if format_main {
            self.print_main_form_separator(text, separator_char, total_width)?;
        } else {
            self.print_nested_form_separator(text, separator_char, total_width)?;
        }

        writeln!(stdout)?;
        stdout.flush()?;
        Ok(())
    }

    /// 打印主表单分割线（3行格式）
    fn print_main_form_separator(
        &self,
        text: &str,
        separator_char: &str,
        total_width: usize,
    ) -> Result<()> {
        let mut stdout = std::io::stdout();
        let text_display_width = text.width();
        let remaining_width = total_width.saturating_sub(text_display_width);
        let left_padding = remaining_width / 2;
        let right_padding = remaining_width - left_padding;

        let separator_line = separator_char.repeat(total_width);
        let text_line = format!(
            "{}{}{}",
            " ".repeat(left_padding),
            text,
            " ".repeat(right_padding)
        );

        writeln!(stdout, "{}", separator_line)?;
        writeln!(stdout, "{}", text_line)?;
        writeln!(stdout, "{}", separator_line)?;
        stdout.flush()?;
        Ok(())
    }

    /// 打印嵌套表单分割线（单行格式）
    fn print_nested_form_separator(
        &self,
        text: &str,
        separator_char: &str,
        total_width: usize,
    ) -> Result<()> {
        let mut stdout = std::io::stdout();
        let text_display_width = text.width();
        let remaining_width = total_width.saturating_sub(text_display_width).saturating_sub(2);
        let left_dashes = remaining_width / 2;
        let right_dashes = remaining_width - left_dashes;

        let separator_line = format!(
            "{}{} {}{}",
            separator_char.repeat(left_dashes),
            " ",
            text,
            " ",
        );
        let separator_line = format!("{}{}", separator_line, separator_char.repeat(right_dashes));

        writeln!(stdout, "{}", separator_line)?;
        stdout.flush()?;
        Ok(())
    }
}

/// Arc 验证器适配器，将 Arc<dyn Validator> 转换为 InputBuilder 可接受的类型
struct ArcValidatorAdapter(
    std::sync::Arc<dyn crate::base::interactive::dialog::Validator + Send + Sync>,
);

impl crate::base::interactive::dialog::Validator for ArcValidatorAdapter {
    fn validate(&self, input: &str) -> std::result::Result<(), String> {
        self.0.validate(input)
    }
}
