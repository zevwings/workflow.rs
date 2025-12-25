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
use rstest::rstest;
use serde::Deserialize;
use workflow::base::http::parser::{JsonParser, ResponseParser, TextParser};

#[derive(Debug, Deserialize, PartialEq)]
struct TestStruct {
    id: u32,
    name: String,
}

/// 测试 JSON 解析器解析有效 JSON（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 JsonParser 能够将有效的 JSON 字节解析为不同的类型。
///
/// ## 测试场景
/// 测试各种有效的 JSON 输入：
/// - 解析为结构体
/// - 解析为 Value
/// - 解析为数组
/// - 解析为嵌套对象
///
/// ## 预期结果
/// - 所有有效的 JSON 都能正确解析
#[rstest]
#[case(br#"{"id": 1, "name": "test"}"#, 200, true)]
#[case(br#"{"key": "value", "number": 42}"#, 200, true)]
#[case(b"[1, 2, 3, 4, 5]", 200, true)]
#[case(br#"{"nested": {"key": "value"}}"#, 200, true)]
fn test_json_parser_parse_with_valid_json_succeeds(
    #[case] json_bytes: &[u8],
    #[case] status_code: u16,
    #[case] should_succeed: bool,
) {
    // Arrange: 准备有效的JSON字节（通过参数提供）

    // Act: 解析JSON
    let result: Result<serde_json::Value, _> = JsonParser::parse(json_bytes, status_code);

    // Assert: 验证解析结果
    assert_eq!(result.is_ok(), should_succeed);
}

/// 测试 JSON 解析器解析为结构体
///
/// ## 测试目的
/// 验证 JsonParser 能够将有效的 JSON 字节解析为 Rust 结构体。
///
/// ## 测试场景
/// 1. 准备有效的 JSON 字节
/// 2. 解析为 TestStruct
/// 3. 验证字段值正确
///
/// ## 预期结果
/// - JSON 被正确解析为结构体，字段值正确
#[test]
fn test_json_parser_parse_with_valid_json_return_result() -> Result<()> {
    // Arrange: 准备有效的JSON字节
    let json_bytes = br#"{"id": 1, "name": "test"}"#;

    // Act: 解析JSON
    let result: TestStruct = JsonParser::parse(json_bytes, 200)?;

    // Assert: 验证解析结果正确
    assert_eq!(result.id, 1);
    assert_eq!(result.name, "test");
    Ok(())
}

/// 测试 JSON 解析器处理边界情况（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 JsonParser 能够正确处理各种边界情况。
///
/// ## 测试场景
/// 测试空响应、空白字符响应、无效 JSON 等边界情况
///
/// ## 预期结果
/// - 空响应和空白字符响应能成功解析
/// - 无效 JSON 返回错误
#[rstest]
#[case(b"", 200, true)] // 空响应
#[case(b"   \n\t  ", 200, true)] // 空白字符
#[case(b"not valid json", 200, false)] // 无效 JSON
fn test_json_parser_parse_with_edge_cases(
    #[case] input: &[u8],
    #[case] status_code: u16,
    #[case] should_succeed: bool,
) {
    // Arrange: 准备边界情况输入（通过参数提供）

    // Act: 尝试解析
    let result: Result<serde_json::Value, _> = JsonParser::parse(input, status_code);

    // Assert: 验证解析结果
    assert_eq!(result.is_ok(), should_succeed);

    // 如果是无效 JSON，验证错误消息
    if !should_succeed {
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(error_msg.contains("Failed to parse JSON") || error_msg.contains("JSON"));
        }
    }
}

/// 测试 JSON 解析器处理错误状态码
///
/// ## 测试目的
/// 验证 JsonParser 即使状态码是错误码，也能尝试解析 JSON。
///
/// ## 测试场景
/// 1. 准备错误状态码的 JSON
/// 2. 解析 JSON
/// 3. 验证解析成功
///
/// ## 预期结果
/// - JSON 被成功解析，即使状态码是错误码
#[test]
fn test_json_parser_parse_with_error_status_parses_json_return_false() -> Result<()> {
    // Arrange: 准备错误状态码的JSON（即使状态码是错误，JSON 解析器也应该尝试解析）
    let json_bytes = br#"{"error": "Not Found"}"#;

    // Act: 解析JSON（使用错误状态码）
    let result: serde_json::Value = JsonParser::parse(json_bytes, 404)?;

    // Assert: 验证解析成功
    assert_eq!(result["error"], "Not Found");
    Ok(())
}

/// 测试文本解析器解析有效文本（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 TextParser 能够将各种文本字节解析为字符串。
///
/// ## 测试场景
/// 测试普通文本、UTF-8 文本、多行文本、Unicode 文本等
///
/// ## 预期结果
/// - 所有有效的文本都能正确解析
#[rstest]
#[case(b"Hello, World!", 200, "Hello, World!", true)]
#[case("你好，世界！".as_bytes(), 200, "你好，世界！", true)]
#[case(b"Line 1\nLine 2\nLine 3", 200, "Line 1\nLine 2\nLine 3", true)]
#[case("测试文本 🚀".as_bytes(), 200, "测试文本 🚀", true)]
#[case(b"", 200, "", true)]
fn test_text_parser_parse_with_various_texts_return_result(
    #[case] text_bytes: &[u8],
    #[case] status_code: u16,
    #[case] expected: &str,
    #[case] should_succeed: bool,
) -> Result<()> {
    // Arrange: 准备文本字节（通过参数提供）

    // Act: 解析文本
    let result = TextParser::parse(text_bytes, status_code);

    // Assert: 验证解析结果
    if should_succeed {
        assert_eq!(result?, expected);
    } else {
        assert!(result.is_err());
    }
    Ok(())
}

/// 测试文本解析器处理错误情况（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 TextParser 能够正确处理各种错误情况。
///
/// ## 测试场景
/// 测试错误状态码、无效 UTF-8 等错误情况
///
/// ## 预期结果
/// - 错误情况返回错误，且错误消息包含相关信息
#[rstest]
#[case(b"Error message", 500, true, "500")] // 错误状态码
#[case(&[0xFF, 0xFE, 0xFD], 200, true, "UTF-8")] // 无效 UTF-8
fn test_text_parser_parse_with_error_cases(
    #[case] text_bytes: &[u8],
    #[case] status_code: u16,
    #[case] should_fail: bool,
    #[case] expected_error_contains: &str,
) {
    // Arrange: 准备错误情况输入（通过参数提供）

    // Act: 尝试解析文本
    let result = TextParser::parse(text_bytes, status_code);

    // Assert: 验证返回错误且错误消息包含预期信息
    assert!(result.is_err() == should_fail);
    if should_fail {
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(error_msg.contains(expected_error_contains));
        }
    }
}

/// 测试 JSON 解析器处理大响应
///
/// ## 测试目的
/// 验证 JsonParser 能够正确处理大型 JSON 响应（超过 200 字符）。
///
/// ## 测试场景
/// 1. 准备大型 JSON 响应
/// 2. 解析 JSON
/// 3. 验证解析结果正确
///
/// ## 预期结果
/// - 大型 JSON 被正确解析
#[test]
fn test_json_parser_parse_with_large_response_parses_correctly_return_result() -> Result<()> {
    // Arrange: 准备大响应（超过 200 字符的预览）
    let large_json = format!(r#"{{"data": "{}"}}"#, "x".repeat(300));

    // Act: 解析大型JSON
    let result: serde_json::Value = JsonParser::parse(large_json.as_bytes(), 200)?;

    // Assert: 验证解析结果正确
    let data_str = result["data"].as_str().expect("data should be a string");
    assert_eq!(data_str.len(), 300);
    Ok(())
}

/// 测试 JSON 解析器错误消息预览功能
///
/// ## 测试目的
/// 验证当 JSON 解析失败时，错误消息中包含预览信息。
///
/// ## 测试场景
/// 1. 准备无效的长 JSON（>200字符）
/// 2. 尝试解析
/// 3. 验证错误消息包含预览信息
///
/// ## 预期结果
/// - 错误消息包含预览信息或 "Failed to parse JSON"
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

/// 测试 JSON 解析器空响应回退机制
///
/// ## 测试目的
/// 验证当空响应无法解析为 null 时，JsonParser 会回退到解析 {}。
///
/// ## 测试场景
/// 1. 准备空响应
/// 2. 尝试解析为需要字段的结构体
/// 3. 验证返回错误且错误消息包含相关信息
///
/// ## 预期结果
/// - 返回错误，错误消息包含 "Failed to parse empty response as JSON"
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

// ==================== Additional Tests from parser_core.rs ====================

/// 测试 JSON 解析器解析数组 JSON
///
/// ## 测试目的
/// 验证 JsonParser 能够将数组格式的 JSON 解析为 Vec。
///
/// ## 测试场景
/// 1. 准备数组 JSON 字节
/// 2. 解析为 Vec<i32>
/// 3. 验证解析结果为数组
///
/// ## 预期结果
/// - JSON 数组被正确解析为 Vec
#[test]
fn test_json_parser_parse_with_array_json_return_result() -> color_eyre::Result<()> {
    // Arrange: 准备数组JSON字节
    let json_bytes = b"[1, 2, 3, 4, 5]";

    // Act: 解析数组JSON
    let result: Vec<i32> = JsonParser::parse(json_bytes, 200)?;

    // Assert: 验证解析结果为数组
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
    Ok(())
}

/// 测试 JSON 解析器解析嵌套对象
///
/// ## 测试目的
/// 验证 JsonParser 能够正确解析嵌套的 JSON 对象。
///
/// ## 测试场景
/// 1. 准备嵌套对象 JSON 字节
/// 2. 解析为 Value
/// 3. 验证嵌套值正确
///
/// ## 预期结果
/// - 嵌套对象被正确解析，嵌套值可访问
#[test]
fn test_json_parser_parse_with_nested_object_return_result() -> color_eyre::Result<()> {
    // Arrange: 准备嵌套对象JSON字节
    let json_bytes = b"{\"nested\": {\"key\": \"value\"}}";

    // Act: 解析嵌套对象JSON
    let result: serde_json::Value = JsonParser::parse(json_bytes, 200)?;

    // Assert: 验证嵌套值正确
    assert_eq!(result["nested"]["key"], "value");
    Ok(())
}

/// 测试文本解析器解析多行文本
///
/// ## 测试目的
/// 验证 TextParser 能够正确处理包含换行符的多行文本。
///
/// ## 测试场景
/// 1. 准备多行文本字节
/// 2. 解析为字符串
/// 3. 验证解析结果正确
///
/// ## 预期结果
/// - 多行文本被正确解析，换行符被保留
#[test]
fn test_text_parser_parse_with_multiline_text_return_result() -> color_eyre::Result<()> {
    // Arrange: 准备多行文本字节
    let text_bytes = b"Line 1\nLine 2\nLine 3";

    // Act: 解析多行文本
    let result = TextParser::parse(text_bytes, 200)?;

    // Assert: 验证解析结果正确
    assert_eq!(result, "Line 1\nLine 2\nLine 3");
    Ok(())
}

/// 测试文本解析器解析 Unicode 文本
///
/// ## 测试目的
/// 验证 TextParser 能够正确处理 Unicode 字符（包括 emoji）。
///
/// ## 测试场景
/// 1. 准备 Unicode 文本字节
/// 2. 解析为字符串
/// 3. 验证解析结果正确
///
/// ## 预期结果
/// - Unicode 文本被正确解析，包括 emoji
#[test]
fn test_text_parser_parse_with_unicode_text_return_result() -> color_eyre::Result<()> {
    // Arrange: 准备Unicode文本字节
    let text_bytes = "测试文本 🚀".as_bytes();

    // Act: 解析Unicode文本
    let result = TextParser::parse(text_bytes, 200)?;

    // Assert: 验证解析结果正确
    assert_eq!(result, "测试文本 🚀");
    Ok(())
}

/// 测试 JSON 解析器解析自定义结构体
///
/// ## 测试目的
/// 验证 JsonParser 能够将 JSON 解析为自定义的 Rust 结构体。
///
/// ## 测试场景
/// 1. 准备自定义结构体和 JSON 字节
/// 2. 解析为自定义结构体
/// 3. 验证字段值正确
///
/// ## 预期结果
/// - JSON 被正确解析为自定义结构体，字段值正确
#[test]
fn test_json_parser_parse_with_custom_struct_return_result() -> color_eyre::Result<()> {
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

/// 测试 JSON 解析器错误消息包含状态码
///
/// ## 测试目的
/// 验证当解析失败时，错误消息中包含 HTTP 状态码信息。
///
/// ## 测试场景
/// 1. 准备长响应（>200字符）且解析失败
/// 2. 使用错误状态码（500）尝试解析
/// 3. 验证错误消息包含状态码
///
/// ## 预期结果
/// - 错误消息包含状态码和 "Failed to parse JSON"
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
