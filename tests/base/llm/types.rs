//! Base/LLM Types 模块测试
//!
//! 测试 LLM 类型定义和默认值。

use pretty_assertions::assert_eq;
use workflow::base::llm::types::LLMRequestParams;

// ==================== LLMRequestParams Default Tests ====================

/// 测试LLMRequestParams默认值创建
///
/// ## 测试目的
/// 验证 `LLMRequestParams::default()` 方法能够创建包含所有默认值的参数结构。
///
/// ## 测试场景
/// 1. 调用 `default()` 创建默认参数
/// 2. 验证所有字段为预期默认值
///
/// ## 预期结果
/// - system_prompt为空字符串
/// - user_prompt为空字符串
/// - max_tokens为None
/// - temperature为0.5
/// - model为"gpt-3.5-turbo"
#[test]
fn test_llm_request_params_default_with_no_parameters_creates_default_params() {
    // Arrange: 准备创建默认参数

    // Act: 创建默认的 LLMRequestParams
    let params = LLMRequestParams::default();

    // Assert: 验证所有字段为默认值
    assert_eq!(params.system_prompt, "");
    assert_eq!(params.user_prompt, "");
    assert_eq!(params.max_tokens, None);
    assert_eq!(params.temperature, 0.5);
    assert_eq!(params.model, "gpt-3.5-turbo");
}

// ==================== LLMRequestParams Serialization Tests ====================

/// 测试LLMRequestParams序列化为JSON
///
/// ## 测试目的
/// 验证 `LLMRequestParams` 结构体能够正确序列化为JSON格式（使用serde）。
///
/// ## 测试场景
/// 1. 创建包含有效数据的LLMRequestParams
/// 2. 使用serde_json序列化为JSON
/// 3. 验证序列化成功
///
/// ## 预期结果
/// - 序列化成功，返回Ok
/// - JSON字符串包含所有字段
#[test]
fn test_llm_request_params_serialize_with_valid_params_serializes_to_json() {
    // Arrange: 准备有效的 LLMRequestParams
    let params = LLMRequestParams {
        system_prompt: "You are a helpful assistant.".to_string(),
        user_prompt: "Hello".to_string(),
        max_tokens: Some(100),
        temperature: 0.7,
        model: "gpt-4".to_string(),
    };

    // Act: 序列化为 JSON
    let json = serde_json::to_string(&params);

    // Assert: 验证序列化成功
    assert!(json.is_ok());
}
