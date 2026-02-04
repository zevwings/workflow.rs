//! 表格渲染模块

mod builder;
mod render;
mod row;
mod tabled;
mod width;

pub use builder::{Alignment, TableBuilder, TableStyle};
pub use tabled::Tabled;

/// 去除 ANSI 转义代码
///
/// 解析字符串并移除所有 ANSI 转义序列（匹配 ESC[ ... m 格式），
/// 返回纯文本内容。用于计算文本的实际显示宽度。
pub(crate) fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // 跳过 ANSI 转义序列
            if chars.peek() == Some(&'[') {
                chars.next(); // 跳过 '['
                              // 跳过数字和分号，直到找到 'm'
                while let Some(&ch) = chars.peek() {
                    if ch == 'm' {
                        chars.next();
                        break;
                    } else if ch.is_ascii_digit() || ch == ';' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// 便捷函数
pub fn table(headers: Vec<impl Into<String>>) -> TableBuilder {
    TableBuilder::new(headers)
}
