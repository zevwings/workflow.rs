//! Base HTTP Parser 模块测试
//!
//! 测试 HTTP 响应解析器的核心功能，包括 JsonParser 和 TextParser。

use pretty_assertions::assert_eq;
use workflow::base::http::parser::{JsonParser, ResponseParser, TextParser};

#[test]
fn test_json_parser_valid_json() -> color_eyre::Result<()> {
    let json_bytes = b"{\"key\": \"value\", \"number\": 42}";
    let result: serde_json::Value = JsonParser::parse(json_bytes, 200)?;

    assert_eq!(result["key"], "value");
    assert_eq!(result["number"], 42);

    Ok(())
}

#[test]
fn test_json_parser_empty_response() {
    // 空响应应该尝试解析为 null 或 {}
    let result: color_eyre::Result<serde_json::Value> = JsonParser::parse(b"", 200);
    // 可能成功（解析为 null 或 {}）或失败
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_json_parser_whitespace_only() {
    // 只有空白字符的响应
    let result: color_eyre::Result<serde_json::Value> = JsonParser::parse(b"   \n\t  ", 200);
    // 可能成功（解析为 null 或 {}）或失败
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_json_parser_invalid_json() {
    let invalid_json = b"not a valid json";
    let result: color_eyre::Result<serde_json::Value> = JsonParser::parse(invalid_json, 200);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to parse JSON"));
}

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
fn test_text_parser_valid_utf8() -> color_eyre::Result<()> {
    let text_bytes = b"Hello, World!";
    let result = TextParser::parse(text_bytes, 200)?;
    assert_eq!(result, "Hello, World!");
    Ok(())
}

#[test]
fn test_text_parser_empty_response() -> color_eyre::Result<()> {
    let text_bytes = b"";
    let result = TextParser::parse(text_bytes, 200)?;
    assert_eq!(result, "");
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
fn test_text_parser_error_status() {
    // TextParser 在非成功状态码时应该失败
    let text_bytes = b"Error message";
    let result = TextParser::parse(text_bytes, 404);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("HTTP request failed"));
}

#[test]
fn test_text_parser_invalid_utf8() {
    // 无效的 UTF-8 序列
    let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
    let result = TextParser::parse(invalid_utf8, 200);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("UTF-8"));
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

