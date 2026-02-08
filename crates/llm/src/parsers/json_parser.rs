//! JSON 响应解析器
//!
//! 提供从 LLM 响应中解析 JSON 的功能。
//!
//! 支持两种解析模式：
//! 1. 泛型模式：直接转换为实现了 `Deserialize` 的 model
//! 2. Map 模式：转换为 `serde_json::Map<String, Value>`（Rust 中 JSON 对象的标准表示）

use crate::LLMError;
use serde::Deserialize;
use serde_json::{Map, Value};

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

        serde_json::from_str(&json_str).map_err(|e| {
            LLMError::ApiError(format!(
                "Failed to parse LLM response as JSON. Raw response: {} - {}",
                json_str, e
            ))
        })
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
