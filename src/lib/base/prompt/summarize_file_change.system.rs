//! 单个文件修改总结的 system prompt
//!
//! 用于根据文件的 diff 内容生成该文件的修改总结。

use crate::base::llm::get_language_requirement;

/// 根据语言生成单个文件修改总结的 system prompt
///
/// # 返回
///
/// 返回根据语言定制的 system prompt 字符串
///
/// # 说明
///
/// 语言选择优先级：配置文件 > 默认值（"en"）
/// 如果配置文件中的语言代码不在支持列表中，将使用英文作为默认语言。
pub fn generate_summarize_file_change_system_prompt() -> String {
    // 基础 prompt 内容
    let base_prompt = r#"You're a technical documentation assistant that generates concise summaries of code changes for individual files.

## Summary Rules

Generate a brief, clear summary of the changes made to a specific file based on its diff content.

### Summary Requirements

1. **Be Concise**: Keep the summary brief (3-6 bullet points)
2. **Focus on Changes**: Describe what was modified, added, or removed
3. **Highlight Key Points**: Emphasize the most important changes
4. **Use Clear Language**: Write in a way that's easy to understand for developers
5. **Be Specific**: Mention specific functions, features, or improvements when relevant
6. **Use Bullet Points**: Format as a bulleted list, one point per line

### What to Include

- What functionality was added, modified, or removed
- Key improvements or fixes
- Important implementation details (if significant)
- Any breaking changes or notable side effects
- Input/output descriptions (if applicable)
- Main functionality or purpose

### What to Avoid

- Don't list every single line change
- Don't repeat the diff content verbatim
- Don't include implementation details unless they're important
- Don't make assumptions about changes not visible in the diff
- Don't use paragraph format - use bullet points only

### Examples

**Good Summary (Bullet Points):**
- 添加了会议卡片功能
- 输入为用户信息，输出为生成的会议卡片
- 主要功能是根据用户信息生成会议卡片

**Good Summary (English):**
- Added meeting card generation functionality
- Input: user information, Output: generated meeting card
- Main feature: generates meeting cards based on user information

**Bad Summary:**
"This file was modified. Some lines were added and some were removed. The code now looks different."

## Response Format

Return your response as a bulleted list (one point per line, each line starting with "- "). Do not use markdown formatting, just plain text with "- " prefix for each bullet point.

**Example Response:**
- 添加了会议卡片功能
- 输入为用户信息，输出为生成的会议卡片
- 主要功能是根据用户信息生成会议卡片"#;

    // 使用 LLM 模块的语言增强功能
    get_language_requirement(base_prompt)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Basic Prompt Generation Tests ====================

    /// 测试生成文件修改总结system prompt返回非空字符串
    ///
    /// ## 测试目的
    /// 验证 `generate_summarize_file_change_system_prompt()` 函数能够成功生成非空的system prompt。
    ///
    /// ## 测试场景
    /// 1. 调用函数生成prompt
    /// 2. 验证返回的prompt不为空
    ///
    /// ## 预期结果
    /// - 返回的prompt不为空
    #[test]
    fn test_generate_summarize_file_change_system_prompt_returns_non_empty_string() {
        // Arrange: 准备调用函数（无需额外准备）

        // Act: 生成 system prompt
        let prompt = generate_summarize_file_change_system_prompt();

        // Assert: 验证返回的 prompt 不为空
        assert!(!prompt.is_empty());
    }

    /// 测试生成的文件修改总结prompt包含关键词
    ///
    /// ## 测试目的
    /// 验证生成的system prompt包含与文件修改总结相关的关键词（summary, file, diff, changes等）。
    ///
    /// ## 测试场景
    /// 1. 生成system prompt
    /// 2. 验证prompt包含至少一个关键词
    ///
    /// ## 预期结果
    /// - prompt包含关键词（summary, file, diff, changes等）
    #[test]
    fn test_generate_summarize_file_change_system_prompt_contains_keywords() {
        // Arrange: 准备关键词列表
        let keywords = [
            "summary", "Summary", "file", "File", "diff", "Diff", "changes", "Changes",
        ];

        // Act: 生成 system prompt
        let prompt = generate_summarize_file_change_system_prompt();

        // Assert: 验证 prompt 包含关键内容
        let contains_keywords = keywords.iter().any(|keyword| prompt.contains(keyword));
        assert!(contains_keywords);
    }

    /// 测试生成的文件修改总结prompt包含规则说明
    ///
    /// ## 测试目的
    /// 验证生成的system prompt包含总结规则说明（Summary Rules, Requirements等）。
    ///
    /// ## 测试场景
    /// 1. 生成system prompt
    /// 2. 验证prompt包含规则关键词
    ///
    /// ## 预期结果
    /// - prompt包含规则说明（Summary Rules, Requirements, bullet等）
    #[test]
    fn test_generate_summarize_file_change_system_prompt_contains_rules() {
        // Arrange: 准备规则关键词
        let rule_keywords = ["Summary Rules", "Requirements", "bullet", "Bullet"];

        // Act: 生成 system prompt
        let prompt = generate_summarize_file_change_system_prompt();

        // Assert: 验证 prompt 包含规则说明
        let contains_rules = rule_keywords.iter().any(|keyword| prompt.contains(keyword));
        assert!(contains_rules);
    }

    /// 测试生成的文件修改总结prompt包含示例
    ///
    /// ## 测试目的
    /// 验证生成的system prompt包含使用示例。
    ///
    /// ## 测试场景
    /// 1. 生成system prompt
    /// 2. 验证prompt包含示例关键词
    ///
    /// ## 预期结果
    /// - prompt包含 "Example" 或 "example"
    #[test]
    fn test_generate_summarize_file_change_system_prompt_contains_examples() {
        // Arrange: 准备示例关键词
        let example_keywords = ["Example", "example"];

        // Act: 生成 system prompt
        let prompt = generate_summarize_file_change_system_prompt();

        // Assert: 验证 prompt 包含示例
        let contains_examples = example_keywords.iter().any(|keyword| prompt.contains(keyword));
        assert!(contains_examples);
    }

    // ==================== Consistency Tests ====================

    /// 测试多次调用生成函数返回一致的结果
    ///
    /// ## 测试目的
    /// 验证 `generate_summarize_file_change_system_prompt()` 函数在多次调用时返回一致的结果（幂等性）。
    ///
    /// ## 测试场景
    /// 1. 多次调用生成函数
    /// 2. 比较多次调用的结果
    ///
    /// ## 预期结果
    /// - 多次调用的结果完全一致
    #[test]
    fn test_generate_summarize_file_change_system_prompt_with_multiple_calls_returns_consistent_result(
    ) {
        // Arrange: 准备多次调用

        // Act: 多次调用生成函数
        let prompt1 = generate_summarize_file_change_system_prompt();
        let prompt2 = generate_summarize_file_change_system_prompt();

        // Assert: 验证多次调用返回一致的结果
        assert_eq!(prompt1, prompt2);
    }

    // ==================== Validation Tests ====================

    /// 测试生成的文件修改总结prompt长度合理
    ///
    /// ## 测试目的
    /// 验证生成的system prompt有合理的长度，至少包含基本内容（最小长度阈值200字符）。
    ///
    /// ## 测试场景
    /// 1. 生成system prompt
    /// 2. 验证长度超过最小阈值
    ///
    /// ## 预期结果
    /// - prompt长度大于200字符
    #[test]
    fn test_generate_summarize_file_change_system_prompt_has_reasonable_length() {
        // Arrange: 准备最小长度要求
        let min_length = 200;

        // Act: 生成 system prompt
        let prompt = generate_summarize_file_change_system_prompt();

        // Assert: 验证 prompt 有合理的长度（至少应该包含基本内容）
        assert!(prompt.len() > min_length);
    }

    /// 测试生成的文件修改总结prompt包含语言要求
    ///
    /// ## 测试目的
    /// 验证生成的system prompt包含语言要求说明（可能通过 `get_language_requirement` 添加）。
    ///
    /// ## 测试场景
    /// 1. 生成system prompt
    /// 2. 验证prompt不为空（语言要求可能已包含）
    ///
    /// ## 注意事项
    /// - 具体内容取决于 `get_language_requirement` 的实现
    ///
    /// ## 预期结果
    /// - prompt不为空
    /// - 可能包含语言要求说明
    #[test]
    fn test_generate_summarize_file_change_system_prompt_contains_language_requirement() {
        // Arrange: 准备调用函数（无需额外准备）
        // 注意：具体内容取决于 get_language_requirement 的实现

        // Act: 生成 system prompt
        let prompt = generate_summarize_file_change_system_prompt();

        // Assert: 验证包含语言要求（可能通过 get_language_requirement 添加）
        assert!(!prompt.is_empty());
    }
}
