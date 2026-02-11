//! LLM 响应 Fixtures
//!
//! 预定义的常见 LLM 输出文本，用于测试提交消息、PR 描述等场景。

/// LLM 响应 Fixtures
///
/// 提供测试中常用的预定义响应字符串，避免在测试里手写大段示例文本。
pub struct LLMFixtures;

impl LLMFixtures {
    /// 示例提交消息生成响应（符合 Conventional Commits）
    pub fn commit_message() -> &'static str {
        "feat: add new user authentication feature\n\n\
         - Implement JWT-based authentication\n\
         - Add user login and registration endpoints\n\
         - Update database schema for user table"
    }

    /// 示例 PR 描述生成响应
    pub fn pr_description() -> &'static str {
        "## Summary\n\
         This PR adds a new user authentication feature.\n\n\
         ## Changes\n\
         - Add JWT library\n\
         - Implement login endpoint\n\
         - Add tests\n\n\
         ## Test Plan\n\
         - Unit tests pass\n\
         - Integration tests pass"
    }

    /// 示例错误或异常时的响应文本（用于测试错误处理分支）
    pub fn error_response() -> &'static str {
        "Error: Unable to generate response"
    }

    /// 短回复（用于测试最小输出）
    pub fn short_response() -> &'static str {
        "OK"
    }

    /// 空内容（用于测试空响应处理）
    pub fn empty_response() -> &'static str {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_message_starts_with_conventional_prefix() {
        let s = LLMFixtures::commit_message();
        assert!(s.starts_with("feat:"));
    }

    #[test]
    fn pr_description_contains_expected_sections() {
        let s = LLMFixtures::pr_description();
        assert!(s.contains("## Summary"));
        assert!(s.contains("## Changes"));
        assert!(s.contains("## Test Plan"));
    }

    #[test]
    fn error_response_is_non_empty() {
        assert!(!LLMFixtures::error_response().is_empty());
    }

    #[test]
    fn short_and_empty_responses_are_consistent() {
        assert_eq!(LLMFixtures::short_response(), "OK");
        assert_eq!(LLMFixtures::empty_response(), "");
    }
}
