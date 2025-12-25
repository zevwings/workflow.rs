//! HTTP Parser 测试
//!
//! 测试 HTTP 响应解析器的功能。
//!
//! ## 测试策略
//!
//! - 所有测试返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 测试 JSON 和文本解析器的各种场景
//! - 测试错误处理和边界情况

use color_eyre::Result;
use serde::Deserialize;
use workflow::base::http::parser::{JsonParser, ResponseParser, TextParser};

#[derive(Debug, Deserialize, PartialEq)]
struct TestStruct {
    id: u32,
    name: String,
}

#[test]
fn test_json_parser_success() -> Result<()> {
    let json_bytes = br#"{"id": 1, "name": "test"}"#;
    let result: TestStruct = JsonParser::parse(json_bytes, 200)?;
    assert_eq!(result.id, 1);
    assert_eq!(result.name, "test");
    Ok(())
}

#[test]
fn test_json_parser_value() -> Result<()> {
    let json_bytes = br#"{"key": "value", "number": 42}"#;
    let result: serde_json::Value = JsonParser::parse(json_bytes, 200)?;
    assert_eq!(result["key"], "value");
    assert_eq!(result["number"], 42);
    Ok(())
}

#[test]
fn test_json_parser_empty_response() {
    // 空响应应该尝试解析为 null 或 {}
    let empty_bytes = b"";
    let result: Result<serde_json::Value, _> = JsonParser::parse(empty_bytes, 200);
    // 应该成功解析为 null 或 {}
    assert!(result.is_ok());
}

#[test]
fn test_json_parser_whitespace_response() {
    // 只有空白字符的响应
    let whitespace_bytes = b"   \n\t  ";
    let result: Result<serde_json::Value, _> = JsonParser::parse(whitespace_bytes, 200);
    // 应该成功解析为 null 或 {}
    assert!(result.is_ok());
}

#[test]
fn test_json_parser_invalid_json() {
    let invalid_bytes = b"not valid json";
    let result: Result<serde_json::Value, _> = JsonParser::parse(invalid_bytes, 200);
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("Failed to parse JSON"));
    }
}

#[test]
fn test_json_parser_error_status() -> Result<()> {
    // 即使状态码是错误，JSON 解析器也应该尝试解析
    let json_bytes = br#"{"error": "Not Found"}"#;
    let result: serde_json::Value = JsonParser::parse(json_bytes, 404)?;
    assert_eq!(result["error"], "Not Found");
    Ok(())
}

#[test]
fn test_text_parser_success() -> Result<()> {
    let text_bytes = b"Hello, World!";
    let result = TextParser::parse(text_bytes, 200)?;
    assert_eq!(result, "Hello, World!");
    Ok(())
}

#[test]
fn test_text_parser_utf8() -> Result<()> {
    let utf8_bytes = "你好，世界！".as_bytes();
    let result = TextParser::parse(utf8_bytes, 200)?;
    assert_eq!(result, "你好，世界！");
    Ok(())
}

#[test]
fn test_text_parser_error_status() {
    // TextParser 应该拒绝非成功状态码
    let text_bytes = b"Error message";
    let result = TextParser::parse(text_bytes, 500);
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("500"));
    }
}

#[test]
fn test_text_parser_invalid_utf8() {
    // 无效的 UTF-8 序列
    let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
    let result = TextParser::parse(invalid_utf8, 200);
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("UTF-8"));
    }
}

#[test]
fn test_text_parser_empty() -> Result<()> {
    let empty_bytes = b"";
    let result = TextParser::parse(empty_bytes, 200)?;
    assert_eq!(result, "");
    Ok(())
}

#[test]
fn test_json_parser_large_response() -> Result<()> {
    // 测试大响应（超过 200 字符的预览）
    let large_json = format!(r#"{{"data": "{}"}}"#, "x".repeat(300));
    let result: serde_json::Value = JsonParser::parse(large_json.as_bytes(), 200)?;
    let data_str = result["data"].as_str().expect("data should be a string");
    assert_eq!(data_str.len(), 300);
    Ok(())
}

#[test]
fn test_json_parser_error_preview() {
    // 测试错误消息中的预览功能
    let invalid_json = format!(r#"{{"invalid": "{}"}}"#, "x".repeat(250));
    // 破坏 JSON 格式
    let broken_json = invalid_json + "invalid";
    let result: Result<serde_json::Value, _> = JsonParser::parse(broken_json.as_bytes(), 200);
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        // 应该包含预览信息
        assert!(error_msg.contains("preview") || error_msg.contains("Failed to parse JSON"));
    }
}

#[test]
fn test_json_parser_empty_response_fallback_to_object() {
    // 测试空响应时，如果解析 null 失败，会回退到解析 {}
    // 使用一个不能从 null 反序列化的类型来触发 or_else 分支
    let empty_bytes = b"";
    // 尝试解析为空对象（需要至少一个字段的结构体）
    // 由于 null 不能反序列化为需要字段的结构体，会触发 or_else 分支
    let result: Result<TestStruct, _> = JsonParser::parse(empty_bytes, 200);
    // 这个应该失败，因为 {} 也没有必需的字段
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        // 应该包含错误消息
        assert!(error_msg.contains("Failed to parse empty response as JSON"));
    }
}

// ==================== 来自 parser_core.rs 的补充测试 ====================

#[test]
fn test_json_parser_array() -> color_eyre::Result<()> {
    let json_bytes = b"[1, 2, 3, 4, 5]";
    let result: Vec<i32> = JsonParser::parse(json_bytes, 200)?;
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
    Ok(())
}

#[test]
fn test_json_parser_nested_object() -> color_eyre::Result<()> {
    let json_bytes = b"{\"nested\": {\"key\": \"value\"}}";
    let result: serde_json::Value = JsonParser::parse(json_bytes, 200)?;
    assert_eq!(result["nested"]["key"], "value");
    Ok(())
}

#[test]
fn test_text_parser_multiline() -> color_eyre::Result<()> {
    let text_bytes = b"Line 1\nLine 2\nLine 3";
    let result = TextParser::parse(text_bytes, 200)?;
    assert_eq!(result, "Line 1\nLine 2\nLine 3");
    Ok(())
}

#[test]
fn test_text_parser_unicode() -> color_eyre::Result<()> {
    let text_bytes = "测试文本 🚀".as_bytes();
    let result = TextParser::parse(text_bytes, 200)?;
    assert_eq!(result, "测试文本 🚀");
    Ok(())
}

#[test]
fn test_json_parser_custom_struct() -> color_eyre::Result<()> {
    #[derive(serde::Deserialize, PartialEq, Debug)]
    struct TestStruct {
        name: String,
        age: u32,
    }

    let json_bytes = b"{\"name\": \"Alice\", \"age\": 30}";
    let result: TestStruct = JsonParser::parse(json_bytes, 200)?;

    assert_eq!(result.name, "Alice");
    assert_eq!(result.age, 30);

    Ok(())
}

#[test]
fn test_json_parser_long_response_with_status() {
    // 测试长响应（>200字符）且解析失败时，status 参数在错误消息中的使用
    let long_invalid_json = format!("{{\"key\": \"value\"{}", "x".repeat(300));
    let result: color_eyre::Result<serde_json::Value> =
        JsonParser::parse(long_invalid_json.as_bytes(), 500);
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("500"));
        assert!(error_msg.contains("Failed to parse JSON"));
    }
}
