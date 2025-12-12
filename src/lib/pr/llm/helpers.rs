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
