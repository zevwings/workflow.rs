//! LLM 客户端测试
//!
//! 测试 LLM 客户端的 JSON 解析功能，支持多种 OpenAI 兼容格式。

use insta::assert_json_snapshot;
use pretty_assertions::assert_eq;

use serde_json::json;
use workflow::base::llm::client::LLMClient;

#[test]
fn test_extract_from_openai_standard() {
    // 测试标准 OpenAI 格式（包含所有必需字段）
    let json = json!({
        "id": "chatcmpl-test",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "gpt-3.5-turbo",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Test content"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15
        }
    });

    let client = LLMClient::global();
    let result = client.extract_content(&json).unwrap();
    assert_eq!(result, "Test content");

    // 使用快照测试验证 JSON 结构
    assert_json_snapshot!("openai_standard_response", json);
}

#[test]
fn test_extract_from_openai_proxy() {
    // 测试 proxy 格式（包含扩展字段，但符合 OpenAI 标准）
    let json = json!({
        "id": "chatcmpl-CfonRS9pFvyJW33Opwz83wHhVIGnz",
        "object": "chat.completion",
        "created": 1764082745,
        "model": "gpt-3.5-turbo-0125",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Test response content",
                "refusal": null,
                "annotations": []
            },
            "logprobs": null,
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 62,
            "completion_tokens": 56,
            "total_tokens": 118,
            "prompt_tokens_details": {
                "cached_tokens": 0,
                "audio_tokens": 0
            },
            "completion_tokens_details": {
                "reasoning_tokens": 0,
                "audio_tokens": 0,
                "accepted_prediction_tokens": 0,
                "rejected_prediction_tokens": 0
            }
        },
        "service_tier": "default",
        "system_fingerprint": null
    });

    let client = LLMClient::global();
    let result = client.extract_content(&json).unwrap();
    assert_eq!(result, "Test response content");

    // 使用快照测试验证 JSON 结构
    assert_json_snapshot!("openai_proxy_response", json);
}

#[test]
fn test_extract_from_cerebras_proxy() {
    // 测试另一种 proxy 格式变体（字段顺序不同，缺少部分扩展字段，但有新的 time_info）
    let json = json!({
        "id": "chatcmpl-97c1fe15-05df-490d-a1b9-8540771db334",
        "choices": [{
            "finish_reason": "stop",
            "index": 0,
            "message": {
                "content": "Test response content",
                "role": "assistant"
            }
        }],
        "created": 1764083329,
        "model": "qwen-3-235b-a22b-instruct-2507",
        "system_fingerprint": "fp_d2d9b827ee854c39818d",
        "object": "chat.completion",
        "usage": {
            "total_tokens": 186,
            "completion_tokens": 123,
            "prompt_tokens": 63
        },
        "time_info": {
            "queue_time": 0.001855552,
            "prompt_time": 0.003562439,
            "completion_time": 0.120426404,
            "total_time": 0.12685751914978027,
            "created": 1764083329.0582647
        }
    });

    let client = LLMClient::global();
    let result = client.extract_content(&json).unwrap();
    assert_eq!(result, "Test response content");

    // 使用快照测试验证 JSON 结构
    assert_json_snapshot!("cerebras_proxy_response", json);
}
