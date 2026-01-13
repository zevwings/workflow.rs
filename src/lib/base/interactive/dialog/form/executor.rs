//! 表单执行器

use crate::base::interactive::config::PromptConfig;
use crate::base::interactive::dialog::form::builder::FormBuilder;
use crate::base::interactive::dialog::form::field::{FieldType, FormField};
use crate::base::interactive::dialog::form::result::FormResult;
use crate::base::interactive::error::{PromptError, Result};
use crate::base::interactive::terminal::Terminal;
use unicode_width::UnicodeWidthStr;

/// 表单执行器
pub struct FormExecutor {
    _config: PromptConfig,
}

impl FormExecutor {
    /// 创建新的表单执行器
    pub fn new(config: PromptConfig) -> Self {
        Self { _config: config }
    }

    /// 执行表单字段序列
    pub fn execute<TR: Terminal>(
        &self,
        builder: &FormBuilder,
        terminal: &mut TR,
    ) -> Result<FormResult> {
        self.execute_with_level(builder, terminal, 0)
    }

    /// 执行表单字段序列（带层级信息）
    fn execute_with_level<TR: Terminal>(
        &self,
        builder: &FormBuilder,
        terminal: &mut TR,
        level: usize,
    ) -> Result<FormResult> {
        let title = builder.get_title();

        // 判断是主表单（level == 0）还是嵌套表单（level > 0）
        let is_main_form = level == 0;

        // 输出开始分割线
        if let Some(title) = title {
            if is_main_form {
                // 主表单：显示开始和结束分割线（带 Start/End 后缀）
                self.print_separator(terminal, title, "start", is_main_form)?;
            } else {
                // 嵌套表单：只显示开始分割线（不带 Start/End 后缀）
                self.print_nested_form_separator_simple(terminal, title)?;
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
            let value = self.execute_field(field, &result, terminal, level)?;

            // 收集结果
            result.set(field.key.clone(), value);
        }

        // 输出结束分割线（仅主表单显示）
        if let Some(title) = title {
            if is_main_form {
                self.print_separator(terminal, title, "end", is_main_form)?;
            }
        }

        Ok(result)
    }

    /// 执行单个字段
    fn execute_field<TR: Terminal>(
        &self,
        field: &FormField,
        _current_result: &FormResult,
        terminal: &mut TR,
        level: usize,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        match field.field_type {
            FieldType::Confirm => self.execute_confirm_field(field, terminal),
            FieldType::Input => self.execute_input_field(field, terminal, false),
            FieldType::Password => self.execute_input_field(field, terminal, true),
            FieldType::Select => self.execute_select_field(field, terminal),
            FieldType::MultiSelect => self.execute_multiselect_field(field, terminal),
            FieldType::Form => self.execute_nested_form(field, terminal, level),
        }
    }

    /// 执行确认字段
    fn execute_confirm_field<TR: Terminal>(
        &self,
        field: &FormField,
        terminal: &mut TR,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let default_value = field
            .default_value
            .as_ref()
            .and_then(|v| v.downcast_ref::<bool>())
            .copied()
            .unwrap_or(false);

        let confirmed = crate::base::interactive::dialog::confirm::confirm(&field.prompt)
            .default(default_value)
            .prompt(terminal)?;

        Ok(Box::new(confirmed))
    }

    /// 执行输入字段（Input 或 Password）
    fn execute_input_field<TR: Terminal>(
        &self,
        field: &FormField,
        terminal: &mut TR,
        is_password: bool,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let default_value = field
            .default_value
            .as_ref()
            .and_then(|v| v.downcast_ref::<String>())
            .cloned()
            .unwrap_or_default();

        let mut builder = if is_password {
            crate::base::interactive::dialog::input::input(&field.prompt).password()
        } else {
            crate::base::interactive::dialog::input::input(&field.prompt)
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

        let value = builder.prompt(terminal)?;
        Ok(Box::new(value))
    }

    /// 执行选择字段
    fn execute_select_field<TR: Terminal>(
        &self,
        field: &FormField,
        terminal: &mut TR,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let default_index = field.default_index.unwrap_or(0);
        let selected =
            crate::base::interactive::dialog::select::select(&field.prompt, field.options.clone())
                .default(default_index)
                .prompt(terminal)?;

        // 找到选中项的索引
        let index = field
            .options
            .iter()
            .enumerate()
            .find(|(_, opt)| *opt == &selected)
            .map(|(idx, _)| idx)
            .unwrap_or(default_index);

        Ok(Box::new(index))
    }

    /// 执行多选字段
    fn execute_multiselect_field<TR: Terminal>(
        &self,
        field: &FormField,
        terminal: &mut TR,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let selected = crate::base::interactive::dialog::multiselect::multiselect(
            &field.prompt,
            field.options.clone(),
        )
        .default(field.default_selected.clone())
        .prompt(terminal)?;

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
    fn execute_nested_form<TR: Terminal>(
        &self,
        field: &FormField,
        terminal: &mut TR,
        level: usize,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let nested_form = field
            .nested_form
            .as_ref()
            .ok_or_else(|| PromptError::InvalidInput("嵌套表单不能为空".to_string()))?;

        let nested_result = self.execute_with_level(nested_form, terminal, level + 1)?;
        Ok(Box::new(nested_result))
    }

    /// 打印分割线
    fn print_separator<TR: Terminal>(
        &self,
        terminal: &mut TR,
        title: &str,
        suffix: &str,
        is_main_form: bool,
    ) -> Result<()> {
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

        self.print_separator_line(
            terminal,
            &text,
            SEPARATOR_CHAR,
            SEPARATOR_LENGTH,
            is_main_form,
        )
    }

    /// 打印嵌套表单分割线（单行格式，不带 Start/End 后缀）
    fn print_nested_form_separator_simple<TR: Terminal>(
        &self,
        terminal: &mut TR,
        title: &str,
    ) -> Result<()> {
        const SEPARATOR_CHAR: &str = "─";
        const SEPARATOR_LENGTH: usize = 72;
        self.print_separator_line(terminal, title, SEPARATOR_CHAR, SEPARATOR_LENGTH, false)
    }

    /// 打印分割线（统一方法）
    fn print_separator_line<TR: Terminal>(
        &self,
        terminal: &mut TR,
        text: &str,
        separator_char: &str,
        total_width: usize,
        format_main: bool,
    ) -> Result<()> {
        terminal.write_flush("\n")?;

        if format_main {
            self.print_main_form_separator(terminal, text, separator_char, total_width)?;
        } else {
            self.print_nested_form_separator(terminal, text, separator_char, total_width)?;
        }

        terminal.write_flush("\n")?;
        Ok(())
    }

    /// 打印主表单分割线（3行格式）
    fn print_main_form_separator<TR: Terminal>(
        &self,
        terminal: &mut TR,
        text: &str,
        separator_char: &str,
        total_width: usize,
    ) -> Result<()> {
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

        terminal.write_flush(&format!("{}\n", separator_line))?;
        terminal.write_flush(&format!("{}\n", text_line))?;
        terminal.write_flush(&format!("{}\n", separator_line))?;
        Ok(())
    }

    /// 打印嵌套表单分割线（单行格式）
    fn print_nested_form_separator<TR: Terminal>(
        &self,
        terminal: &mut TR,
        text: &str,
        separator_char: &str,
        total_width: usize,
    ) -> Result<()> {
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

        terminal.write_flush(&format!("{}\n", separator_line))?;
        Ok(())
    }
}

/// Arc 验证器适配器，将 Arc<dyn Validator> 转换为 InputBuilder 可接受的类型
struct ArcValidatorAdapter(
    std::sync::Arc<dyn crate::base::interactive::dialog::input::Validator + Send + Sync>,
);

impl crate::base::interactive::dialog::input::Validator for ArcValidatorAdapter {
    fn validate(&self, input: &str) -> std::result::Result<(), String> {
        self.0.validate(input)
    }
}
