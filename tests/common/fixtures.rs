//! 测试数据和 Fixtures
//!
//! 提供测试中使用的测试数据和固定数据。

/// 测试用的 Jira Issue JSON 响应示例
pub const JIRA_ISSUE_JSON: &str = r#"{
    "key": "PROJ-123",
    "fields": {
        "summary": "Test Issue",
        "description": "This is a test issue",
        "status": {
            "name": "In Progress"
        },
        "assignee": {
            "displayName": "Test User",
            "emailAddress": "test@example.com"
        }
    }
}"#;

/// 测试用的 GitHub PR JSON 响应示例
pub const GITHUB_PR_JSON: &str = r#"{
    "number": 123,
    "title": "Test PR",
    "body": "This is a test PR",
    "state": "open",
    "user": {
        "login": "testuser"
    }
}"#;

/// 测试用的 LLM 响应 JSON 示例
pub const LLM_RESPONSE_JSON: &str = r#"{
    "id": "chatcmpl-test",
    "object": "chat.completion",
    "created": 1234567890,
    "model": "gpt-3.5-turbo",
    "choices": [{
        "index": 0,
        "message": {
            "role": "assistant",
            "content": "Test response"
        },
        "finish_reason": "stop"
    }],
    "usage": {
        "prompt_tokens": 10,
        "completion_tokens": 5,
        "total_tokens": 15
    }
}"#;
