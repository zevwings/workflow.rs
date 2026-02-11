//! Mock LLM 客户端
//!
//! 不发送真实 API 请求，按预定义顺序返回响应，用于单元测试与集成测试。

use std::sync::{Arc, Mutex};

use crate::{
    client::{LLMClient, LLMRequestParameters},
    LLMError,
};

/// Mock LLM 客户端
///
/// 不发送真实的 API 请求，而是按添加顺序循环返回预定义的响应。
/// 可用于测试提交消息生成、PR 描述生成等场景，无需依赖外部 LLM 服务。
pub struct MockLLMClient {
    /// 预定义响应列表，按调用顺序循环使用
    responses: Arc<Mutex<Vec<String>>>,
    /// 已调用次数
    call_count: Arc<Mutex<usize>>,
}

impl MockLLMClient {
    /// 创建新的 Mock 客户端（无预定义响应时，调用将返回错误）
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(Vec::new())),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    /// 添加一条预定义响应
    ///
    /// 多次调用时，将按添加顺序依次返回；若响应用尽则循环使用最后一条。
    pub fn add_response(&self, response: impl Into<String>) {
        self.responses.lock().unwrap().push(response.into());
    }

    /// 获取已调用次数
    pub fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    /// 清空预定义响应并重置调用计数（便于在同一测试中复用）
    pub fn reset(&self) {
        *self.responses.lock().unwrap() = Vec::new();
        *self.call_count.lock().unwrap() = 0;
    }
}

impl LLMClient for MockLLMClient {
    fn call(&self, _params: &LLMRequestParameters) -> Result<String, LLMError> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;
        let call_index = *count;

        let responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            return Err(LLMError::Other(
                "No response configured for mock. Use add_response() to set expected responses."
                    .to_string(),
            ));
        }
        let index = (call_index - 1) % responses.len();
        let content = responses[index].clone();
        Ok(content)
    }
}

impl Default for MockLLMClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::LLMFixtures;

    #[test]
    fn mock_client_returns_added_responses() {
        let client = MockLLMClient::new();
        client.add_response(LLMFixtures::commit_message());

        let params = LLMRequestParameters::default();
        let content = client.call(&params).unwrap();

        assert!(content.starts_with("feat:"));
        assert_eq!(client.call_count(), 1);
    }

    #[test]
    fn mock_client_cycles_through_multiple_responses() {
        let client = MockLLMClient::new();
        client.add_response("first");
        client.add_response("second");

        let params = LLMRequestParameters::default();
        assert_eq!(client.call(&params).unwrap(), "first");
        assert_eq!(client.call(&params).unwrap(), "second");
        assert_eq!(client.call(&params).unwrap(), "first");
        assert_eq!(client.call_count(), 3);
    }

    #[test]
    fn mock_client_errors_when_no_responses_configured() {
        let client = MockLLMClient::new();
        let params = LLMRequestParameters::default();

        let err = client.call(&params).unwrap_err();
        assert!(matches!(err, LLMError::Other(_)));
        assert_eq!(client.call_count(), 1);
    }

    #[test]
    fn mock_client_reset_clears_responses_and_count() {
        let client = MockLLMClient::new();
        client.add_response("before");
        let _ = client.call(&LLMRequestParameters::default());
        assert_eq!(client.call_count(), 1);

        client.reset();
        assert_eq!(client.call_count(), 0);
        let err = client.call(&LLMRequestParameters::default()).unwrap_err();
        assert!(matches!(err, LLMError::Other(_)));
    }
}
