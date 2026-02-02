//! 分割线渲染函数

use crate::dialog::Result;
use std::io::Write;
use unicode_width::UnicodeWidthStr;

/// 打印分割线
pub(super) fn print_separator(title: &str, suffix: &str, is_main_form: bool) -> Result<()> {
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

    print_separator_line(&text, SEPARATOR_CHAR, SEPARATOR_LENGTH, is_main_form)
}

/// 打印嵌套表单分割线（单行格式，不带 Start/End 后缀）
pub(super) fn print_nested_form_separator_simple(title: &str) -> Result<()> {
    const SEPARATOR_CHAR: &str = "─";
    const SEPARATOR_LENGTH: usize = 72;
    print_separator_line(title, SEPARATOR_CHAR, SEPARATOR_LENGTH, false)
}

/// 打印分割线（统一方法）
fn print_separator_line(
    text: &str,
    separator_char: &str,
    total_width: usize,
    format_main: bool,
) -> Result<()> {
    let mut stdout = std::io::stdout();
    writeln!(stdout)?;
    stdout.flush()?;

    if format_main {
        print_main_form_separator(text, separator_char, total_width)?;
    } else {
        print_nested_form_separator(text, separator_char, total_width)?;
    }

    writeln!(stdout)?;
    stdout.flush()?;
    Ok(())
}

/// 打印主表单分割线（3行格式）
fn print_main_form_separator(text: &str, separator_char: &str, total_width: usize) -> Result<()> {
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
fn print_nested_form_separator(text: &str, separator_char: &str, total_width: usize) -> Result<()> {
    let mut stdout = std::io::stdout();
    let text_display_width = text.width();
    let remaining_width = total_width
        .saturating_sub(text_display_width)
        .saturating_sub(2);
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
