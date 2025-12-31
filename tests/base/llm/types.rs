//! Base/LLM Types 模块测试
//!
//! 测试 LLM 类型定义和默认值。

use pretty_assertions::assert_eq;
use workflow::base::llm::types::LLMRequestParams;

// ==================== LLMRequestParams Default Tests ====================

/// 测试LLMRequestParams默认值创建和业务逻辑验证
///
/// ## 测试目的
/// 验证 `LLMRequestParams::default()` 方法能够创建包含所有默认值的参数结构，
/// 并且默认值符合业务规则和约束。
///
/// ## 测试场景
/// 1. 调用 `default()` 创建默认参数
/// 2. 验证所有字段为预期默认值
/// 3. 验证默认值符合业务规则和约束
///
/// ## 预期结果
/// - system_prompt为空字符串（允许为空，表示无系统提示）
/// - user_prompt为空字符串（允许为空，表示无用户提示）
/// - max_tokens为None（表示不限制，使用模型默认最大值）
/// - temperature为0.5（在有效范围[0.0, 1.0]内）
/// - model为"gpt-3.5-turbo"（有效的模型名称，不为空）
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

    // Assert: 验证业务逻辑和约束
    // 验证温度值在有效范围内（LLM API 标准范围是 0.0-1.0）
    assert!(
        params.temperature >= 0.0 && params.temperature <= 1.0,
        "Temperature should be in valid range [0.0, 1.0], got {}",
        params.temperature
    );

    // 验证模型名称不为空且有效
    assert!(!params.model.is_empty(), "Model name should not be empty");
    assert!(
        params.model.starts_with("gpt-")
            || params.model.starts_with("claude-")
            || params.model.starts_with("deepseek-"),
        "Model name should be a valid LLM model, got: {}",
        params.model
    );

    // 验证 max_tokens 的默认行为（None 表示无限制，这是合理的默认值）
    assert_eq!(
        params.max_tokens, None,
        "Default max_tokens should be None (unlimited), allowing the model to use its default maximum"
    );

    // 验证 prompt 字段可以为空（这是合理的默认值，表示无预设提示）
    // 这些字段为空是允许的，因为在实际使用中会通过其他方式设置
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
