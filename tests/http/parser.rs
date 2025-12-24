//! HTTP Parser 测试
//!
//! 测试 HTTP 响应解析器的功能。

use serde::Deserialize;
use workflow::base::http::parser::{JsonParser, ResponseParser, TextParser};

#[derive(Debug, Deserialize, PartialEq)]
struct TestStruct {
    id: u32,
    name: String,
}

#[test]
fn test_json_parser_success() {
    let json_bytes = br#"{"id": 1, "name": "test"}"#;
    let result: TestStruct = JsonParser::parse(json_bytes, 200).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.name, "test");
}

#[test]
fn test_json_parser_value() {
    let json_bytes = br#"{"key": "value", "number": 42}"#;
    let result: serde_json::Value = JsonParser::parse(json_bytes, 200).unwrap();
    assert_eq!(result["key"], "value");
    assert_eq!(result["number"], 42);
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
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to parse JSON"));
}

#[test]
fn test_json_parser_error_status() {
    // 即使状态码是错误，JSON 解析器也应该尝试解析
    let json_bytes = br#"{"error": "Not Found"}"#;
    let result: Result<serde_json::Value, _> = JsonParser::parse(json_bytes, 404);
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["error"], "Not Found");
}

#[test]
fn test_text_parser_success() {
    let text_bytes = b"Hello, World!";
    let result = TextParser::parse(text_bytes, 200).unwrap();
    assert_eq!(result, "Hello, World!");
}

#[test]
fn test_text_parser_utf8() {
    let utf8_bytes = "你好，世界！".as_bytes();
    let result = TextParser::parse(utf8_bytes, 200).unwrap();
    assert_eq!(result, "你好，世界！");
}

#[test]
fn test_text_parser_error_status() {
    // TextParser 应该拒绝非成功状态码
    let text_bytes = b"Error message";
    let result = TextParser::parse(text_bytes, 500);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("500"));
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
fn test_text_parser_empty() {
    let empty_bytes = b"";
    let result = TextParser::parse(empty_bytes, 200).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_json_parser_large_response() {
    // 测试大响应（超过 200 字符的预览）
    let large_json = format!(r#"{{"data": "{}"}}"#, "x".repeat(300));
    let result: Result<serde_json::Value, _> = JsonParser::parse(large_json.as_bytes(), 200);
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["data"].as_str().unwrap().len(), 300);
}

#[test]
fn test_json_parser_error_preview() {
    // 测试错误消息中的预览功能
    let invalid_json = format!(r#"{{"invalid": "{}"}}"#, "x".repeat(250));
    // 破坏 JSON 格式
    let broken_json = invalid_json + "invalid";
    let result: Result<serde_json::Value, _> = JsonParser::parse(broken_json.as_bytes(), 200);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    // 应该包含预览信息
    assert!(error_msg.contains("preview") || error_msg.contains("Failed to parse JSON"));
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
    let error_msg = result.unwrap_err().to_string();
    // 应该包含错误消息
    assert!(error_msg.contains("Failed to parse empty response as JSON"));
}
