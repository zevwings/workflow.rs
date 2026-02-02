//! Spinner 格式化工具
//!
//! 提供格式化 spinner 文本的功能。

use crate::style::theme::Theme;

/// 格式化 spinner 文本
pub(crate) fn format_spinner_text(frame: &str, message: &str, theme: &Theme) -> String {
    if message.is_empty() {
        theme.spinner.apply(frame, theme.enable_color)
    } else {
        let spinner_part = theme.spinner.apply(frame, theme.enable_color);
        let message_part = theme.spinner.apply(message, theme.enable_color);
        format!("{} {}", spinner_part, message_part)
    }
}
