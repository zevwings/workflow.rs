//! JSON 响应解析器
//!
//! 提供从 LLM 响应中解析 JSON 的功能。
//!
//! 支持两种解析模式：
//! 1. 泛型模式：直接转换为实现了 `Deserialize` 的 model
//! 2. Map 模式：转换为 `serde_json::Map<String, Value>`（Rust 中 JSON 对象的标准表示）

use serde::Deserialize;
use serde_json::{Map, Value};

use crate::LLMError;

/// JSON 解析模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonParseMode {
    /// 原始字符串，直接解析
    Raw,
    /// 从 markdown 代码块中提取 JSON 后解析
    ExtractFromMarkdown,
}

/// JSON 响应解析器
///
/// 负责从 LLM 响应中提取和解析 JSON 数据。
pub struct JsonParser;

impl JsonParser {
    /// 从 LLM 响应中提取 JSON 字符串
    ///
    /// 支持处理多种 LLM 响应格式：
    /// - ````json\n{...}\n````（标准 markdown 代码块）
    /// - ````JSON\n{...}\n````（大写语言标签）
    /// - ````\n{...}\n````（无语言标签的代码块）
    /// - `Some text\n```json\n{...}\n````（代码块前有说明文字）
    /// - ````json\n{...}\n````\nSome text`（代码块后有说明文字）
    /// - ````json\n{...}`（无闭合标记）
    /// - 纯 JSON 字符串
    ///
    /// # 参数
    ///
    /// * `response` - LLM 返回的响应字符串（可能是 JSON 或包含 JSON 的 markdown 代码块）
    ///
    /// # 返回
    ///
    /// 返回提取的 JSON 字符串（已去除 markdown 代码块包装）
    pub fn extract_json(response: impl AsRef<str>) -> String {
        let text = response.as_ref().trim();

        // 在整个文本中查找 markdown 代码块的开始位置（支持 ```json、```JSON、```）
        let fence_start = text
            .find("```json")
            .or_else(|| text.find("```JSON"))
            .or_else(|| text.find("```"));

        let Some(fence_pos) = fence_start else {
            // 没有找到代码块，原样返回
            return text.to_string();
        };

        // 从代码块开始位置之后找到第一个换行符，内容从换行符之后开始
        let after_fence = &text[fence_pos..];
        let content_start = match after_fence.find('\n') {
            Some(p) => fence_pos + p + 1,
            None => return text.to_string(),
        };

        // 从内容开始位置之后查找闭合的 ```
        let content_end = text[content_start..]
            .find("```")
            .map(|p| content_start + p)
            .unwrap_or(text.len());

        text[content_start..content_end].trim().to_string()
    }

    /// 解析 JSON 字符串为 Value
    ///
    /// # 参数
    ///
    /// * `json_str` - JSON 字符串
    /// * `mode` - 解析模式，决定是否从 markdown 代码块中提取 JSON
    ///
    /// # 返回
    ///
    /// 返回解析后的 JSON Value
    ///
    /// # 错误
    ///
    /// 如果 JSON 格式不正确，返回相应的错误信息。
    pub fn parse(json_str: impl AsRef<str>, mode: JsonParseMode) -> Result<Value, LLMError> {
        let json_str = match mode {
            JsonParseMode::Raw => json_str.as_ref().to_string(),
            JsonParseMode::ExtractFromMarkdown => Self::extract_json(json_str),
        };

        // 首先尝试直接解析
        match serde_json::from_str(&json_str) {
            Ok(value) => Ok(value),
            Err(original_error) => {
                // 如果解析失败，尝试修复常见的 JSON 错误
                let fixed_json = Self::try_fix_json(&json_str);

                // 尝试解析修复后的 JSON
                serde_json::from_str(&fixed_json).map_err(|_| {
                    // 如果修复后仍然失败，返回原始错误
                    LLMError::ApiError(format!(
                        "Failed to parse LLM response as JSON. Raw response: {} - {}",
                        json_str, original_error
                    ))
                })
            }
        }
    }

    /// 从响应中提取并解析 JSON 为指定的模型类型
    ///
    /// 支持直接将 JSON 响应转换为实现了 `Deserialize` 的 model。
    ///
    /// # 类型参数
    ///
    /// * `T` - 目标类型，必须实现 `Deserialize<'de>`
    ///
    /// # 参数
    ///
    /// * `response` - LLM 返回的响应字符串
    ///
    /// # 返回
    ///
    /// 返回解析后的指定类型实例
    ///
    /// # 错误
    ///
    /// 如果 JSON 格式不正确或无法转换为目标类型，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// // `JsonParser` 位于 storage 的内部 LLM 模块中（非公共 API），此示例仅用于说明用法。
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct MyModel {
    ///     field1: String,
    ///     field2: Option<String>,
    /// }
    ///
    /// let response = r#"{"field1": "value", "field2": "optional"}"#.to_string();
    /// let model: MyModel = JsonParser::to_model(response)?;
    /// ```
    pub fn to_model<T>(response: impl AsRef<str>) -> Result<T, LLMError>
    where
        T: for<'de> Deserialize<'de>,
    {
        // 先提取并解析为 Value，然后转换为目标类型
        let mut value = Self::parse(response, JsonParseMode::ExtractFromMarkdown)?;
        // LLM 可能返回 null 值（如 "old_value": null），
        // 递归移除 null 值的字段，让 #[serde(default)] 提供默认值
        Self::remove_null_values(&mut value);
        serde_json::from_value(value).map_err(|e| {
            LLMError::ApiError(format!(
                "Failed to convert JSON Value to type {}: {}",
                std::any::type_name::<T>(),
                e
            ))
        })
    }

    /// 从响应中提取并解析 JSON 为 Map（Map 模式）
    ///
    /// 将 JSON 对象转换为 `serde_json::Map<String, Value>`，适用于需要动态访问 JSON 字段的场景。
    ///
    /// # 参数
    ///
    /// * `response` - LLM 返回的响应字符串
    ///
    /// # 返回
    ///
    /// 返回解析后的 JSON Map
    ///
    /// # 错误
    ///
    /// 如果 JSON 格式不正确或不是对象类型，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// // `JsonParser` 位于 storage 的内部 LLM 模块中（非公共 API），此示例仅用于说明用法。
    /// let response = r#"{"branch_name": "add-feature", "pr_title": "Add feature"}"#.to_string();
    /// let map = JsonParser::to_map(response)?;
    /// let branch_name = map.get("branch_name").and_then(|v| v.as_str());
    /// ```
    /// 尝试修复 LLM 响应中常见的 JSON 格式错误
    ///
    /// 修复的常见问题：
    /// 1. 缺失的开头引号：`"key": value"` -> `"key": "value"`
    /// 2. 尾部逗号：`[1, 2,]` -> `[1, 2]`、`{"a": 1,}` -> `{"a": 1}`
    ///
    /// # 参数
    ///
    /// * `json_str` - 可能格式错误的 JSON 字符串
    ///
    /// # 返回
    ///
    /// 修复后的 JSON 字符串
    fn try_fix_json(json_str: &str) -> String {
        let mut fixed = json_str.to_string();

        // 修复 1: 缺失的开头引号
        // 匹配模式: "key": <非引号字符开头的内容>"
        // 例如: "purpose": 将项目文档统一至..." -> "purpose": "将项目文档统一至..."
        fixed = Self::fix_missing_opening_quotes(&fixed);

        // 修复 2: 移除尾部逗号
        // 例如: [1, 2,] -> [1, 2]  或  {"a": 1,} -> {"a": 1}
        fixed = Self::remove_trailing_commas(&fixed);

        fixed
    }

    /// 修复缺失的开头引号
    ///
    /// 查找形如 `"key": value"` 的模式，并将其修复为 `"key": "value"`
    fn fix_missing_opening_quotes(json_str: &str) -> String {
        let mut result = String::with_capacity(json_str.len());
        let chars = json_str.chars().peekable();
        let mut in_string = false;
        let mut after_colon = false;
        let mut pending_quote = false;

        for ch in chars {
            match ch {
                '"' => {
                    in_string = !in_string;
                    after_colon = false;
                    pending_quote = false;
                    result.push(ch);
                }
                ':' if !in_string => {
                    after_colon = true;
                    result.push(ch);
                }
                // 如果在冒号后遇到空白，跳过
                ' ' | '\t' | '\n' | '\r' if after_colon && !in_string => {
                    result.push(ch);
                }
                // 如果在冒号后遇到非引号、非空白、非特殊字符的内容，说明缺少开头引号
                _ if after_colon && !in_string && !pending_quote => {
                    // 检查是否是 JSON 特殊字符（数字、布尔值、null、数组、对象）
                    if ch.is_ascii_digit()
                        || ch == '-'
                        || ch == '{'
                        || ch == '['
                        || ch == 't'
                        || ch == 'f'
                        || ch == 'n'
                    {
                        // 这些是合法的 JSON 值开始字符，不需要添加引号
                        result.push(ch);
                        after_colon = false;
                    } else {
                        // 不是特殊字符，说明这是一个字符串值但缺少开头引号
                        result.push('"');
                        result.push(ch);
                        pending_quote = true;
                        after_colon = false;
                    }
                }
                _ => {
                    result.push(ch);
                    if pending_quote && ch == '"' {
                        pending_quote = false;
                    }
                }
            }
        }

        result
    }

    /// 移除尾部逗号
    ///
    /// 移除数组和对象中的尾部逗号，例如：
    /// - `[1, 2,]` -> `[1, 2]`
    /// - `{"a": 1,}` -> `{"a": 1}`
    fn remove_trailing_commas(json_str: &str) -> String {
        let mut result = String::with_capacity(json_str.len());
        let chars: Vec<char> = json_str.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if ch == ',' {
                // 查找逗号后的第一个非空白字符
                let mut j = i + 1;
                while j < chars.len() && matches!(chars[j], ' ' | '\t' | '\n' | '\r') {
                    j += 1;
                }

                // 如果逗号后是 ] 或 }，说明这是尾部逗号，跳过它
                if j < chars.len() && matches!(chars[j], ']' | '}') {
                    // 跳过逗号，但保留空白
                    i += 1;
                    continue;
                }
            }

            result.push(ch);
            i += 1;
        }

        result
    }

    /// 递归移除 JSON 对象中值为 null 的字段
    ///
    /// LLM 响应中可能包含 `"field": null` 的情况，但目标结构体的字段类型是 `String`
    /// 而非 `Option<String>`。`#[serde(default)]` 只在字段缺失时生效，不处理显式的 null 值。
    /// 通过移除 null 值的字段，让 `#[serde(default)]` 对这些字段提供默认值。
    fn remove_null_values(value: &mut Value) {
        match value {
            Value::Object(map) => {
                map.retain(|_, v| !v.is_null());
                for v in map.values_mut() {
                    Self::remove_null_values(v);
                }
            }
            Value::Array(arr) => {
                for v in arr.iter_mut() {
                    Self::remove_null_values(v);
                }
            }
            _ => {}
        }
    }

    pub fn to_map(response: impl AsRef<str>) -> Result<Map<String, Value>, LLMError> {
        let value = Self::parse(response, JsonParseMode::ExtractFromMarkdown)?;

        match value {
            Value::Object(map) => Ok(map),
            _ => Err(LLMError::ApiError(format!(
                "Expected JSON object, but got: {}",
                value
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_missing_opening_quote() {
        // 测试缺失开头引号的情况
        let broken_json = r#"{"purpose": 将项目文档统一至 `docs/guidelines`"}"#;
        let fixed = JsonParser::fix_missing_opening_quotes(broken_json);
        assert_eq!(
            fixed,
            r#"{"purpose": "将项目文档统一至 `docs/guidelines`"}"#
        );
    }

    #[test]
    fn test_fix_missing_opening_quote_with_english() {
        let broken_json = r#"{"name": John Doe"}"#;
        let fixed = JsonParser::fix_missing_opening_quotes(broken_json);
        assert_eq!(fixed, r#"{"name": "John Doe"}"#);
    }

    #[test]
    fn test_dont_break_valid_json_values() {
        // 测试不应修改合法的 JSON 值
        let valid_json = r#"{"count": 42, "active": true, "data": null, "items": [1, 2]}"#;
        let fixed = JsonParser::fix_missing_opening_quotes(valid_json);
        assert_eq!(fixed, valid_json);
    }

    #[test]
    fn test_remove_trailing_commas_in_array() {
        let broken_json = r#"{"items": [1, 2, 3,]}"#;
        let fixed = JsonParser::remove_trailing_commas(broken_json);
        assert_eq!(fixed, r#"{"items": [1, 2, 3]}"#);
    }

    #[test]
    fn test_remove_trailing_commas_in_object() {
        let broken_json = r#"{"name": "test", "count": 5,}"#;
        let fixed = JsonParser::remove_trailing_commas(broken_json);
        assert_eq!(fixed, r#"{"name": "test", "count": 5}"#);
    }

    #[test]
    fn test_remove_trailing_commas_with_whitespace() {
        let broken_json = "{\n  \"items\": [1, 2,  ],\n  \"count\": 5,\n}";
        let fixed = JsonParser::remove_trailing_commas(broken_json);
        assert_eq!(fixed, "{\n  \"items\": [1, 2  ],\n  \"count\": 5\n}");
    }

    #[test]
    fn test_try_fix_json_combined() {
        // 测试同时修复缺失引号和尾部逗号
        let broken_json = r#"{"purpose": 测试内容", "items": [1, 2,]}"#;
        let fixed = JsonParser::try_fix_json(broken_json);
        assert_eq!(fixed, r#"{"purpose": "测试内容", "items": [1, 2]}"#);
    }

    #[test]
    fn test_parse_with_auto_fix() {
        // 测试完整的解析流程，包括自动修复
        let broken_json = r#"{"title": 测试", "count": 42, "tags": ["a", "b",]}"#;
        let result = JsonParser::parse(broken_json, JsonParseMode::Raw);

        assert!(result.is_ok(), "Should successfully parse and fix JSON");
        let value = result.unwrap();
        assert_eq!(value["title"], "测试");
        assert_eq!(value["count"], 42);
        assert_eq!(value["tags"][0], "a");
        assert_eq!(value["tags"][1], "b");
    }

    #[test]
    fn test_real_world_case_from_error() {
        // 测试实际错误案例中的问题
        let broken_json = r#"{
  "domain": "文档指南迁移与补充",
  "purpose": 将项目文档统一至 `docs/guidelines`，并提供覆盖率监控与测试最佳实践指南",
  "files": ["docs/guidelines/architecture.md"]
}"#;

        let result = JsonParser::parse(broken_json, JsonParseMode::Raw);
        assert!(
            result.is_ok(),
            "Should successfully parse and fix real-world case"
        );
        let value = result.unwrap();
        assert_eq!(value["domain"], "文档指南迁移与补充");
        assert!(value["purpose"].as_str().unwrap().contains("docs/guidelines"));
    }
}
