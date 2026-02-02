//! 字符串截断扩展 trait
//!
//! 为字符串类型提供截断相关的扩展方法。

/// 字符串截断 trait
///
/// 为字符串类型提供截断功能，类似于 Swift extension。
/// 在字符边界安全截取，并尝试在最后一个换行符处截断，以保持格式的完整性。
///
/// # 示例
///
/// ```
/// use toolkit::Truncate;
///
/// let text = "这是一个很长的文本...";
/// let truncated = text.truncate(10, "... (共 {} 字符)");
/// ```
pub trait Truncate {
    /// 截断字符串内容，避免超过指定长度限制
    ///
    /// 在字符边界安全截取，并尝试在最后一个换行符处截断，以保持格式的完整性。
    ///
    /// # 参数
    ///
    /// * `max_length` - 最大字符数
    /// * `truncation_suffix_template` - 截断时添加的后缀消息模板（包含 `{}` 占位符，用于显示总字符数）
    ///
    /// # 返回
    ///
    /// 返回截断后的字符串内容（如果超过最大长度，会添加截断消息）
    ///
    /// # 示例
    ///
    /// ```
    /// use toolkit::Truncate;
    ///
    /// let text = "这是一个很长的文本，需要被截断";
    /// let result = text.truncate(10, "... (共 {} 字符)");
    /// assert!(result.contains("... (共"));
    /// ```
    fn truncate(&self, max_length: usize, truncation_suffix_template: &str) -> String;
}

/// 为 `str` 实现 `Truncate` trait
impl Truncate for str {
    fn truncate(&self, max_length: usize, truncation_suffix_template: &str) -> String {
        let char_count = self.chars().count();
        if char_count <= max_length {
            return self.to_string();
        }

        // 使用字符边界安全截取
        let mut char_boundary = self.len();
        for (idx, _) in self.char_indices().take(max_length + 1) {
            char_boundary = idx;
        }
        let truncated = &self[..char_boundary];

        // 尝试在最后一个换行符处截断，保持格式完整性
        let last_newline = truncated.rfind('\n').unwrap_or(0);
        let truncated_text = if last_newline > 0 {
            &self[..last_newline]
        } else {
            truncated
        };

        let suffix = truncation_suffix_template.replace("{}", &char_count.to_string());
        format!("{}{}", truncated_text, suffix)
    }
}

/// 为 `String` 实现 `Truncate` trait
impl Truncate for String {
    fn truncate(&self, max_length: usize, truncation_suffix_template: &str) -> String {
        self.as_str()
            .truncate(max_length, truncation_suffix_template)
    }
}

#[cfg(test)]
mod tests {
    use super::Truncate;

    #[test]
    fn test_truncate_short_text() {
        let text = "short";
        let result = text.truncate(10, "... (共 {} 字符)");
        assert_eq!(result, "short");
    }

    #[test]
    fn test_truncate_long_text() {
        let text = "这是一个很长的文本，需要被截断";
        let result = text.truncate(10, "... (共 {} 字符)");
        // 截断后的内容应该包含后缀
        assert!(result.contains("... (共"));
        // 截断后的内容应该以截断的文本开头
        assert!(result.starts_with("这是一个很长的"));
    }

    #[test]
    fn test_truncate_with_newline() {
        let text = "line1\nline2\nline3";
        let result = text.truncate(10, "... (共 {} 字符)");
        // 应该在换行符处截断
        assert!(result.contains("line1"));
    }

    #[test]
    fn test_truncate_string_type() {
        let text = String::from("这是一个很长的文本");
        let result = text.truncate(5, "... (共 {} 字符)");
        assert!(result.contains("... (共"));
    }
}
