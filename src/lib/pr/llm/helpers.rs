//! 辅助工具函数
//!
//! 提供 LLM 响应解析的公共方法。

/// 从 markdown 代码块中提取 JSON 字符串（公共方法）
///
/// 支持以下格式：
/// - ````json\n{...}\n````
/// - ````\n{...}\n````
/// - 纯 JSON 字符串
///
/// # 参数
///
/// * `response` - 可能包含 markdown 代码块的响应字符串
///
/// # 返回
///
/// 返回提取的 JSON 字符串（已去除 markdown 代码块包装）
pub fn extract_json_from_markdown(response: String) -> String {
    let trimmed = response.trim();

    // 尝试提取 JSON（可能包含 markdown 代码块）
    if trimmed.starts_with("```json") {
        // 移除 ```json 开头和 ``` 结尾
        let start = trimmed.find('\n').unwrap_or(0);
        let end = trimmed.rfind("```").unwrap_or(trimmed.len());
        trimmed[start..end].trim().to_string()
    } else if trimmed.starts_with("```") {
        // 移除 ``` 开头和 ``` 结尾
        let start = trimmed.find('\n').unwrap_or(0);
        let end = trimmed.rfind("```").unwrap_or(trimmed.len());
        trimmed[start..end].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

/// 修复 JSON 字符串中的转义问题
///
/// LLM 生成的 JSON 可能包含未转义的反斜杠（特别是在 Windows 路径中），
/// 这会导致 JSON 解析失败。此函数尝试修复这些转义问题。
///
/// # 参数
///
/// * `json_str` - 需要修复的 JSON 字符串
///
/// # 返回
///
/// 返回修复后的 JSON 字符串
fn fix_json_escapes(json_str: &str) -> String {
    let mut result = String::with_capacity(json_str.len() * 2);
    let mut chars = json_str.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if !escape_next => {
                in_string = !in_string;
                result.push(ch);
            }
            '\\' if in_string && !escape_next => {
                // 检查下一个字符是否是有效的转义序列
                match chars.peek() {
                    Some(&next) => {
                        // 检查是否是有效的转义字符
                        let is_valid_escape =
                            matches!(next, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' | 'u');
                        if is_valid_escape {
                            // 有效的转义序列，保留原样
                            result.push(ch);
                            escape_next = true;
                        } else if next.is_ascii() {
                            // 无效的转义序列（如 \s, \d），需要转义反斜杠
                            result.push('\\');
                            result.push('\\');
                            // 下一个字符会正常处理（不设置 escape_next）
                        } else {
                            result.push(ch);
                        }
                    }
                    None => {
                        // 字符串末尾的反斜杠，需要转义
                        result.push('\\');
                        result.push('\\');
                    }
                }
            }
            _ => {
                if escape_next {
                    escape_next = false;
                }
                result.push(ch);
            }
        }
    }

    result
}

/// 从 markdown 代码块中提取并修复 JSON 字符串
///
/// 这是 `extract_json_from_markdown` 的增强版本，会自动修复 JSON 字符串中的转义问题。
///
/// # 参数
///
/// * `response` - 可能包含 markdown 代码块的响应字符串
///
/// # 返回
///
/// 返回提取并修复后的 JSON 字符串
pub fn extract_and_fix_json(response: String) -> String {
    let extracted = extract_json_from_markdown(response);
    fix_json_escapes(&extracted)
}
