//! Base/LLM Types 模块测试
//!
//! 测试 LLM 类型定义和默认值。

use pretty_assertions::assert_eq;
use workflow::base::llm::types::LLMRequestParams;

#[test]
fn test_llm_request_params_default() {
    // 测试 LLMRequestParams 的默认值
    let params = LLMRequestParams::default();

    assert_eq!(params.system_prompt, "");
    assert_eq!(params.user_prompt, "");
    assert_eq!(params.max_tokens, None);
    assert_eq!(params.temperature, 0.5);
    assert_eq!(params.model, "gpt-3.5-turbo");
}

#[test]
fn test_llm_request_params_serialize() {
    // 测试 LLMRequestParams 的序列化
    let params = LLMRequestParams {
        system_prompt: "You are a helpful assistant.".to_string(),
        user_prompt: "Hello".to_string(),
        max_tokens: Some(100),
        temperature: 0.7,
        model: "gpt-4".to_string(),
    };

    // 验证可以序列化（不会 panic）
    let json = serde_json::to_string(&params);
    assert!(json.is_ok());
}
