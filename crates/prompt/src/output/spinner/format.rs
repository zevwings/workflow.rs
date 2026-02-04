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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::theme::get_theme;

    #[test]
    fn test_format_spinner_text() {
        let mut theme = get_theme();
        theme.enable_color = false;

        // 带消息
        let result = format_spinner_text("⠋", "Loading", &theme);
        assert!(result.contains("⠋"));
        assert!(result.contains("Loading"));

        // 空消息
        let result = format_spinner_text("⠋", "", &theme);
        assert!(result.contains("⠋"));
        assert!(!result.contains(" "));

        // Unicode 消息
        let result = format_spinner_text("⠋", "正在处理...", &theme);
        assert!(result.contains("正在处理..."));

        // 颜色禁用时无 ANSI 转义码
        assert!(!result.contains("\x1b["));
    }
}
