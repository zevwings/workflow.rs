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
fn test_json_parser_parse_with_valid_json_returns_struct() -> Result<()> {
    // Arrange: 准备有效的JSON字节
    let json_bytes = br#"{"id": 1, "name": "test"}"#;

    // Act: 解析JSON
    let result: TestStruct = JsonParser::parse(json_bytes, 200)?;

    // Assert: 验证解析结果正确
    assert_eq!(result.id, 1);
    assert_eq!(result.name, "test");
    Ok(())
}

#[test]
fn test_json_parser_parse_with_valid_json_returns_value() -> Result<()> {
    // Arrange: 准备有效的JSON字节
    let json_bytes = br#"{"key": "value", "number": 42}"#;

    // Act: 解析JSON为Value
    let result: serde_json::Value = JsonParser::parse(json_bytes, 200)?;

    // Assert: 验证解析结果正确
    assert_eq!(result["key"], "value");
    assert_eq!(result["number"], 42);
    Ok(())
}

#[test]
fn test_json_parser_parse_with_empty_response_handles_gracefully() {
    // Arrange: 准备空响应（空响应应该尝试解析为 null 或 {}）
    let empty_bytes = b"";

    // Act: 尝试解析空响应
    let result: Result<serde_json::Value, _> = JsonParser::parse(empty_bytes, 200);

    // Assert: 应该成功解析为 null 或 {}
    assert!(result.is_ok());
}

#[test]
fn test_json_parser_parse_with_whitespace_response_handles_gracefully() {
    // Arrange: 准备只有空白字符的响应
    let whitespace_bytes = b"   \n\t  ";

    // Act: 尝试解析空白字符响应
    let result: Result<serde_json::Value, _> = JsonParser::parse(whitespace_bytes, 200);

    // Assert: 应该成功解析为 null 或 {}
    assert!(result.is_ok());
}

#[test]
fn test_json_parser_parse_with_invalid_json_returns_error() {
    // Arrange: 准备无效的JSON字节
    let invalid_bytes = b"not valid json";

    // Act: 尝试解析无效JSON
    let result: Result<serde_json::Value, _> = JsonParser::parse(invalid_bytes, 200);

    // Assert: 验证返回错误且错误消息包含"Failed to parse JSON"
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("Failed to parse JSON"));
    }
}

#[test]
fn test_json_parser_parse_with_error_status_parses_json() -> Result<()> {
    // Arrange: 准备错误状态码的JSON（即使状态码是错误，JSON 解析器也应该尝试解析）
    let json_bytes = br#"{"error": "Not Found"}"#;

    // Act: 解析JSON（使用错误状态码）
    let result: serde_json::Value = JsonParser::parse(json_bytes, 404)?;

    // Assert: 验证解析成功
    assert_eq!(result["error"], "Not Found");
    Ok(())
}

#[test]
fn test_text_parser_parse_with_valid_text_returns_string() -> Result<()> {
    // Arrange: 准备有效的文本字节
    let text_bytes = b"Hello, World!";

    // Act: 解析文本
    let result = TextParser::parse(text_bytes, 200)?;

    // Assert: 验证解析结果正确
    assert_eq!(result, "Hello, World!");
    Ok(())
}

#[test]
fn test_text_parser_parse_with_utf8_text_returns_string() -> Result<()> {
    // Arrange: 准备UTF-8文本字节
    let utf8_bytes = "你好，世界！".as_bytes();

    // Act: 解析UTF-8文本
    let result = TextParser::parse(utf8_bytes, 200)?;

    // Assert: 验证解析结果正确
    assert_eq!(result, "你好，世界！");
    Ok(())
}

#[test]
fn test_text_parser_parse_with_error_status_returns_error() {
    // Arrange: 准备错误状态码的文本（TextParser 应该拒绝非成功状态码）
    let text_bytes = b"Error message";

    // Act: 尝试解析文本（使用错误状态码）
    let result = TextParser::parse(text_bytes, 500);

    // Assert: 验证返回错误且错误消息包含状态码
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("500"));
    }
}

#[test]
fn test_text_parser_parse_with_invalid_utf8_returns_error() {
    // Arrange: 准备无效的 UTF-8 序列
    let invalid_utf8 = &[0xFF, 0xFE, 0xFD];

    // Act: 尝试解析无效UTF-8
    let result = TextParser::parse(invalid_utf8, 200);

    // Assert: 验证返回错误且错误消息包含UTF-8相关信息
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("UTF-8"));
    }
}

#[test]
fn test_text_parser_parse_with_empty_bytes_returns_empty_string() -> Result<()> {
    // Arrange: 准备空字节
    let empty_bytes = b"";

    // Act: 解析空文本
    let result = TextParser::parse(empty_bytes, 200)?;

    // Assert: 验证返回空字符串
    assert_eq!(result, "");
    Ok(())
}

#[test]
fn test_json_parser_parse_with_large_response_parses_correctly() -> Result<()> {
    // Arrange: 准备大响应（超过 200 字符的预览）
    let large_json = format!(r#"{{"data": "{}"}}"#, "x".repeat(300));

    // Act: 解析大型JSON
    let result: serde_json::Value = JsonParser::parse(large_json.as_bytes(), 200)?;

    // Assert: 验证解析结果正确
    let data_str = result["data"].as_str().expect("data should be a string");
    assert_eq!(data_str.len(), 300);
    Ok(())
}

#[test]
fn test_json_parser_parse_with_invalid_json_returns_error_with_preview() {
    // Arrange: 准备无效的JSON（测试错误消息中的预览功能）
    let invalid_json = format!(r#"{{"invalid": "{}"}}"#, "x".repeat(250));
    // 破坏 JSON 格式
    let broken_json = invalid_json + "invalid";

    // Act: 尝试解析无效JSON
    let result: Result<serde_json::Value, _> = JsonParser::parse(broken_json.as_bytes(), 200);

    // Assert: 验证返回错误且错误消息包含预览信息
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        // 应该包含预览信息
        assert!(error_msg.contains("preview") || error_msg.contains("Failed to parse JSON"));
    }
}

#[test]
fn test_json_parser_parse_with_empty_response_falls_back_to_object() {
    // Arrange: 准备空响应（测试空响应时，如果解析 null 失败，会回退到解析 {}）
    // 使用一个不能从 null 反序列化的类型来触发 or_else 分支
    let empty_bytes = b"";

    // Act: 尝试解析为空对象（需要至少一个字段的结构体）
    // 由于 null 不能反序列化为需要字段的结构体，会触发 or_else 分支
    let result: Result<TestStruct, _> = JsonParser::parse(empty_bytes, 200);

    // Assert: 验证返回错误且错误消息包含相关信息（这个应该失败，因为 {} 也没有必需的字段）
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        // 应该包含错误消息
        assert!(error_msg.contains("Failed to parse empty response as JSON"));
    }
}

// ==================== 来自 parser_core.rs 的补充测试 ====================

#[test]
fn test_json_parser_parse_with_array_json_returns_array() -> color_eyre::Result<()> {
    // Arrange: 准备数组JSON字节
    let json_bytes = b"[1, 2, 3, 4, 5]";

    // Act: 解析数组JSON
    let result: Vec<i32> = JsonParser::parse(json_bytes, 200)?;

    // Assert: 验证解析结果为数组
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
    Ok(())
}

#[test]
fn test_json_parser_parse_with_nested_object_returns_nested_value() -> color_eyre::Result<()> {
    // Arrange: 准备嵌套对象JSON字节
    let json_bytes = b"{\"nested\": {\"key\": \"value\"}}";

    // Act: 解析嵌套对象JSON
    let result: serde_json::Value = JsonParser::parse(json_bytes, 200)?;

    // Assert: 验证嵌套值正确
    assert_eq!(result["nested"]["key"], "value");
    Ok(())
}

#[test]
fn test_text_parser_parse_with_multiline_text_returns_multiline_string() -> color_eyre::Result<()> {
    // Arrange: 准备多行文本字节
    let text_bytes = b"Line 1\nLine 2\nLine 3";

    // Act: 解析多行文本
    let result = TextParser::parse(text_bytes, 200)?;

    // Assert: 验证解析结果正确
    assert_eq!(result, "Line 1\nLine 2\nLine 3");
    Ok(())
}

#[test]
fn test_text_parser_parse_with_unicode_text_returns_unicode_string() -> color_eyre::Result<()> {
    // Arrange: 准备Unicode文本字节
    let text_bytes = "测试文本 🚀".as_bytes();

    // Act: 解析Unicode文本
    let result = TextParser::parse(text_bytes, 200)?;

    // Assert: 验证解析结果正确
    assert_eq!(result, "测试文本 🚀");
    Ok(())
}

#[test]
fn test_json_parser_parse_with_custom_struct_returns_struct() -> color_eyre::Result<()> {
    // Arrange: 准备自定义结构体和JSON字节
    #[derive(serde::Deserialize, PartialEq, Debug)]
    struct TestStruct {
        name: String,
        age: u32,
    }
    let json_bytes = b"{\"name\": \"Alice\", \"age\": 30}";

    // Act: 解析JSON为自定义结构体
    let result: TestStruct = JsonParser::parse(json_bytes, 200)?;

    // Assert: 验证解析结果正确
    assert_eq!(result.name, "Alice");
    assert_eq!(result.age, 30);

    Ok(())
}

#[test]
fn test_json_parser_parse_with_long_response_and_status_returns_error_with_status() {
    // Arrange: 准备长响应（>200字符）且解析失败时，status 参数在错误消息中的使用
    let long_invalid_json = format!("{{\"key\": \"value\"{}", "x".repeat(300));

    // Act: 尝试解析无效JSON（使用500状态码）
    let result: color_eyre::Result<serde_json::Value> =
        JsonParser::parse(long_invalid_json.as_bytes(), 500);

    // Assert: 验证返回错误且错误消息包含状态码
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("500"));
        assert!(error_msg.contains("Failed to parse JSON"));
    }
}
