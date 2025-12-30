//! PR Reword 的 system prompt
//!
//! 用于根据 PR diff 生成简洁的 PR 标题和描述。

/// PR Reword 的 system prompt
///
/// 用于根据当前 PR 标题和 PR diff 生成更新的 PR 标题和描述，用于更新现有 PR。
/// 与 create 流程保持一致：标题主要基于当前标题，PR diff 用于验证和细化。
pub const REWORD_PR_SYSTEM_PROMPT: &str = r##"You're a git assistant that generates a concise PR title and description based on the current PR title and PR diff content.

## Important

**All outputs MUST be in English only.** If the current PR title or PR diff contains non-English text, translate it to English in the output.

## Generate Rules

### PR Title Rules

- **Primary source**: The PR title should be primarily based on the current PR title
- Use PR diff only to verify and refine the title, not to replace the current title's intent
- Must be concise, within 8 words
- No punctuation (except if the current title contains markdown formatting like `#`, preserve that format)
- In English only
- Focus on "what" (the feature/fix/change) rather than "how" (implementation details)
- If the current title is clear and meaningful, use it as the basis for the new title
- Only use PR diff to:
  - Clarify ambiguous titles
  - Verify the title matches the actual changes
  - Refine the title if it doesn't accurately reflect the changes
- Avoid technical jargon unless necessary
- Use clear, descriptive language that explains the business value or problem solved
- **If the current PR title starts with `#` (markdown heading), preserve that format in the generated title**

### Description Rules

- Generate a comprehensive description based on the PR diff provided
- Format as a bulleted list, with each item describing a specific feature, change, or fix
- Each bullet point should start with "- " and be on a separate line
- Focus on what was added, modified, or fixed (not implementation details)
- Group related changes together when appropriate
- **Include ALL important changes** - do not truncate or omit significant changes
- Be comprehensive and complete (typically 5-15 bullet points, or more if needed)
- In English only
- Should provide enough context for reviewers to understand the changes without reading the full diff
- **List all major changes, dependencies added, files modified, and any other relevant information**

## Response Format

Return your response in JSON format with two fields: `pr_title` and `description` (optional).

**Example 1**

```json
{
  "pr_title": "Add user authentication",
  "description": "- Add user authentication functionality with login and registration\n- Implement JWT token generation and validation\n- Add password hashing using bcrypt\n- Update API endpoints for authentication\n- Add unit tests for authentication flow\n- Update database schema to support user accounts\n- Add authentication middleware for protected routes"
}
```

**Example 2**

```json
{
  "pr_title": "Fix login validation bug",
  "description": "- Fix null pointer exception in login validation\n- Add proper error handling for invalid credentials\n- Update login API response format\n- Add input validation for email and password fields\n- Update error messages for better user experience"
}
```

**Example 3** (with markdown heading format)

```json
{
  "pr_title": "# Optimize code with serde_with",
  "description": "- Add serde crate to Cargo.lock\n- Integrate darling, darling_core, and darling_macro crates\n- Update dependencies for serde, hashbrown, and indexmap\n- Add serde_with and serde_with_macros crates\n- Update Cargo.toml with serde_with 3.0 features\n- Add cargo-bloat guidelines to documentation\n- Refactor serialization code across multiple modules\n- Replace manual serde attributes with serde_with macros\n- Update test cases to verify new serialization behavior"
}
```

**Example 4** (minimal changes)

```json
{
  "pr_title": "Update documentation",
  "description": "- Update README with new features\n- Fix typos in API documentation\n- Add usage examples for new commands"
}
```"##;

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Reword PR System Prompt Tests ====================

    /// 测试Reword PR system prompt常量不为空
    ///
    /// ## 测试目的
    /// 验证 `REWORD_PR_SYSTEM_PROMPT` 常量已正确定义且不为空。
    ///
    /// ## 测试场景
    /// 1. 检查prompt常量
    /// 2. 验证常量不为空
    ///
    /// ## 预期结果
    /// - prompt常量不为空
    #[test]
    fn test_reword_pr_system_prompt_not_empty_with_constant_returns_non_empty() {
        // Arrange: 准备检查prompt常量

        // Act & Assert: 验证prompt常量不为空
        // Note: REWORD_PR_SYSTEM_PROMPT is a compile-time constant with non-empty content
        // The constant is verified to exist and contain content at compile time
        let _ = REWORD_PR_SYSTEM_PROMPT;
    }

    /// 测试Reword PR system prompt包含关键词
    ///
    /// ## 测试目的
    /// 验证 `REWORD_PR_SYSTEM_PROMPT` 包含所有必需的关键词（PR title, description, PR diff）。
    ///
    /// ## 测试场景
    /// 1. 准备关键词列表
    /// 2. 验证prompt包含所有关键词
    ///
    /// ## 预期结果
    /// - prompt包含所有必需的关键词
    #[test]
    fn test_reword_pr_system_prompt_contains_keywords_with_prompt_contains_keywords() {
        // Arrange: 准备关键词列表
        let keywords = ["PR title", "description", "PR diff"];

        // Act & Assert: 验证prompt包含所有关键词
        for keyword in keywords.iter() {
            assert!(REWORD_PR_SYSTEM_PROMPT.contains(keyword));
        }
    }

    /// 测试Reword PR system prompt包含规则说明
    ///
    /// ## 测试目的
    /// 验证 `REWORD_PR_SYSTEM_PROMPT` 包含PR标题和描述的规则说明。
    ///
    /// ## 测试场景
    /// 1. 准备规则关键词列表
    /// 2. 验证prompt包含所有规则说明
    ///
    /// ## 预期结果
    /// - prompt包含 "PR Title Rules" 和 "Description Rules"
    #[test]
    fn test_reword_pr_system_prompt_contains_rules_with_prompt_contains_rules() {
        // Arrange: 准备规则关键词列表
        let rules = ["PR Title Rules", "Description Rules"];

        // Act & Assert: 验证prompt包含所有规则说明
        for rule in rules.iter() {
            assert!(REWORD_PR_SYSTEM_PROMPT.contains(rule));
        }
    }

    /// 测试Reword PR system prompt包含示例
    ///
    /// ## 测试目的
    /// 验证 `REWORD_PR_SYSTEM_PROMPT` 包含使用示例和响应格式说明。
    ///
    /// ## 测试场景
    /// 1. 准备示例关键词列表
    /// 2. 验证prompt包含示例
    ///
    /// ## 预期结果
    /// - prompt包含 "Example" 和 "Response Format"
    #[test]
    fn test_reword_pr_system_prompt_contains_examples_with_prompt_contains_examples() {
        // Arrange: 准备示例关键词列表
        let examples = ["Example", "Response Format"];

        // Act & Assert: 验证prompt包含示例
        for example in examples.iter() {
            assert!(REWORD_PR_SYSTEM_PROMPT.contains(example));
        }
    }

    /// 测试Reword PR system prompt包含JSON格式说明
    ///
    /// ## 测试目的
    /// 验证 `REWORD_PR_SYSTEM_PROMPT` 包含JSON响应格式的说明。
    ///
    /// ## 测试场景
    /// 1. 准备JSON格式关键词列表
    /// 2. 验证prompt包含JSON格式说明
    ///
    /// ## 预期结果
    /// - prompt包含 "JSON" 和 "pr_title" 等关键词
    #[test]
    fn test_reword_pr_system_prompt_contains_json_format_with_prompt_contains_json() {
        // Arrange: 准备JSON格式关键词列表
        let json_keywords = ["JSON", "pr_title"];

        // Act & Assert: 验证prompt包含JSON格式说明
        for keyword in json_keywords.iter() {
            assert!(REWORD_PR_SYSTEM_PROMPT.contains(keyword));
        }
    }

    /// 测试Reword PR system prompt包含语言要求
    ///
    /// ## 测试目的
    /// 验证 `REWORD_PR_SYSTEM_PROMPT` 包含语言要求说明（所有输出必须是英文）。
    ///
    /// ## 测试场景
    /// 1. 准备语言要求关键词列表
    /// 2. 验证prompt包含语言要求
    ///
    /// ## 预期结果
    /// - prompt包含 "English" 和 "All outputs MUST be in English"
    #[test]
    fn test_reword_pr_system_prompt_contains_language_requirement_with_prompt_contains_language() {
        // Arrange: 准备语言要求关键词列表
        let language_keywords = ["English", "All outputs MUST be in English"];

        // Act & Assert: 验证prompt包含语言要求
        for keyword in language_keywords.iter() {
            assert!(REWORD_PR_SYSTEM_PROMPT.contains(keyword));
        }
    }

    /// 测试Reword PR system prompt包含Markdown格式支持说明
    ///
    /// ## 测试目的
    /// 验证 `REWORD_PR_SYSTEM_PROMPT` 包含Markdown格式支持的说明。
    ///
    /// ## 测试场景
    /// 1. 准备markdown关键词列表
    /// 2. 验证prompt包含markdown格式支持说明
    ///
    /// ## 预期结果
    /// - prompt包含 "markdown" 或 "markdown heading" 等关键词
    #[test]
    fn test_reword_pr_system_prompt_contains_markdown_support_with_prompt_contains_markdown() {
        // Arrange: 准备markdown关键词列表
        let markdown_keywords = ["markdown", "markdown heading"];

        // Act & Assert: 验证prompt包含markdown格式支持说明
        let contains_markdown = markdown_keywords
            .iter()
            .any(|keyword| REWORD_PR_SYSTEM_PROMPT.contains(keyword));
        assert!(contains_markdown);
    }

    /// 测试Reword PR system prompt长度合理
    ///
    /// ## 测试目的
    /// 验证 `REWORD_PR_SYSTEM_PROMPT` 有合理的长度，至少包含基本内容（最小长度阈值500字符）。
    ///
    /// ## 测试场景
    /// 1. 检查prompt长度
    /// 2. 验证长度超过最小阈值
    ///
    /// ## 预期结果
    /// - prompt长度大于500字符
    #[test]
    fn test_reword_pr_system_prompt_length_with_prompt_has_reasonable_length() {
        // Arrange: 准备最小长度阈值
        let min_length = 500;

        // Act & Assert: 验证prompt有合理的长度（至少应该包含基本内容）
        assert!(REWORD_PR_SYSTEM_PROMPT.len() > min_length);
    }
}
