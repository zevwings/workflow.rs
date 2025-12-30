//! PR 总结的 system prompt
//!
//! 用于根据 PR 的 diff 内容生成总结文档。

use crate::base::llm::get_language_requirement;

/// 根据语言生成 PR 总结的 system prompt
///
/// # 返回
///
/// 返回根据语言定制的 system prompt 字符串
///
/// # 说明
///
/// 语言选择优先级：配置文件 > 默认值（"en"）
/// 如果配置文件中的语言代码不在支持列表中，将使用英文作为默认语言。
pub fn generate_summarize_pr_system_prompt() -> String {
    // 获取 JSON 响应示例
    let summarize_response_example =     "{
      \"summary\": \"# Add User Authentication\\n\\n## Overview\\nThis PR adds user authentication functionality to the application.\\n\\n## Requirements Analysis\\n\\n### Business Requirements\\nDevelopers need a secure way to authenticate users...\\n\\n### Functional Requirements\\nThe system accepts user credentials and returns authentication tokens...\\n\\n## Key Changes\\n- Added login endpoint\\n- Implemented JWT token generation\\n\\n## Files Changed\\n- `src/auth/login.ts`: Added login handler\\n- `src/auth/jwt.ts`: Added token generation\\n\\n## Technical Details\\nImplemented JWT-based authentication:\\n\\n```typescript\\nfunction generateToken(user: User): string {\\n  return jwt.sign({ userId: user.id }, secret);\\n}\\n```\\n\\n## Testing\\nAdded unit tests for authentication flow.\\n\\n## Usage Instructions\\nRun `npm run test` to execute tests.\",
      \"filename\": \"add-user-authentication\"
    }"
        .to_string();
    // 基础 prompt 内容（使用 format! 宏直接插入 JSON 示例）
    let base_prompt = format!(
        r#"You're a technical documentation assistant that generates comprehensive PR summaries and appropriate filenames based on PR diff content.

## Summary Document Rules

Generate a comprehensive Markdown document that includes:

1. **PR Title**: The title of the Pull Request (as a level 1 heading: # PR Title)
2. **Overview**: A brief summary of what this PR does (2-3 sentences)
3. **Requirements Analysis**: Provide a comprehensive analysis from a requirements perspective. Be specific and detailed:
   - **Business Requirements**:
     - What specific business problem or pain point does this PR solve?
     - Why is this change needed now? What was the motivation?
     - What value does this bring to users or the project?
     - Example: "Developers need a way to quickly understand PR changes without manually reviewing code diffs. This feature automates documentation generation, saving time and ensuring consistency."
   - **Functional Requirements**:
     - What specific features or capabilities are added/modified? List each feature clearly.
     - What are the inputs and outputs?
     - What are the key behaviors or functionalities?
     - Example: "The command accepts a PR ID (or auto-detects from current branch), fetches PR diff, uses LLM to generate summary, and saves to a structured file path."
   - **User Scenarios**:
     - Who are the target users? (e.g., developers, QA, project managers)
     - In what specific scenarios will they use this feature?
     - What are the step-by-step use cases?
     - How will they benefit? What problems does it solve for them?
     - Example: "Developers can run 'workflow pr summarize' after creating a PR to generate documentation. QA teams can use the summary to understand changes before testing. Project managers can review PR summaries for project documentation."
   - **Impact Analysis**:
     - List all affected modules, files, and features specifically
     - What existing functionality is modified?
     - What new components are introduced?
     - Are there any side effects or dependencies on other parts of the system?
     - Example: "Affects: PR command module (new summarize subcommand), PR platform abstraction (new get_pull_request_diff method), LLM service (new summarize_pr method), prompt system (new prompt file)."
   - **Change Categories**:
     - Type: New Feature / Bug Fix / Refactoring / Performance / Documentation / Other (choose the most appropriate)
     - Scope: Frontend / Backend / Infrastructure / Tooling / API / Configuration (can be multiple)
     - Priority: Core Feature / Supporting Feature / Enhancement / Maintenance
   - **Dependencies**:
     - New dependencies added (if any, list package names and versions if applicable)
     - API/Interface changes (if any, describe what changed)
     - Breaking changes (if any, describe what breaks and migration path)
     - Example: "No new external dependencies. Adds new method to PlatformProvider trait (backward compatible)."
4. **Key Changes**: List the main changes made in this PR
   - Use bullet points
   - Focus on functionality, not implementation details
   - Group related changes together
5. **Files Changed**: List all modified files with brief descriptions
   - Format: `- path/to/file.ext`: Description of changes
6. **Technical Details**: Important implementation notes and technical considerations
   - Analyze the code changes to identify technical aspects
   - Architecture changes (if any)
   - Performance improvements or considerations (if any)
   - Breaking changes (if any, clearly state "No breaking changes" if none)
   - Dependencies added/removed (if any, clearly state "No dependency changes" if none)
   - Key design decisions or patterns used
   - Important implementation details that reviewers should know
   - If no significant technical details are found, still provide a brief analysis of the implementation approach
   - DO NOT simply write "No significant technical details provided" - always analyze the code changes
7. **Testing**: Testing approach or test coverage
   - Analyze the code changes to determine testing needs
   - Unit tests added/modified (if any, clearly state if none)
   - Integration tests (if any, clearly state if none)
   - Manual testing steps or scenarios
   - Test coverage information (if available)
   - If no tests are found in the changes, suggest what should be tested or state "No tests included in this PR"
   - DO NOT simply write "No specific testing details provided" - always provide testing guidance based on the changes
8. **Usage Instructions**: Provide clear, actionable instructions on how to use the new feature or changes
   - Command syntax and examples (if it's a new command)
   - Required configuration or setup (if any)
   - Step-by-step usage guide
   - Example outputs or expected results
   - Common use cases
   - If this PR adds a new command, include: "Usage: `workflow [command] [options]`. [Description of what the command does]"
   - If this PR modifies existing functionality, describe how the changes affect usage
   - Always include this section, even if it's brief

## Filename Rules

Generate a concise, descriptive filename based on the PR content:
- Must be a valid filename (no special characters except hyphens and underscores)
- Use lowercase with hyphens to separate words
- Should be descriptive but concise (under 50 characters)
- Include the main feature or fix name
- Example formats:
  - `add-user-authentication`
  - `fix-login-bug`
  - `refactor-api-structure`
  - `update-dependencies`

## Response Format

Return your response in JSON format with two fields: `summary` and `filename`.

The `summary` field should contain a complete Markdown document. Use newline characters (\n) in the JSON string to represent line breaks.

The `filename` field should be a valid filename without extension (e.g., "add-user-authentication").

**CRITICAL: JSON Escaping Rules**

You MUST return a valid JSON object with exactly two fields: summary and filename.

All special characters in the summary field MUST be properly escaped according to JSON standards:
- Newlines: Represent each line break as the two characters: backslash followed by lowercase n
- Double quotes: Represent each quote mark as backslash followed by quote mark
- Backslashes: Represent each backslash as two backslashes
- Backticks: Backticks in markdown code blocks do not need escaping in JSON strings

**Response Structure:**

Your response must be a valid JSON object with this exact structure:

Field 1: summary (string type)
- Contains the complete Markdown document as a single JSON string value
- All markdown content must be within this one string field
- Every line break in your markdown must be written as: backslash + n (two characters)
- Every double quote in your markdown must be written as: backslash + quote (two characters)
- Every backslash in your markdown must be written as: backslash + backslash (two characters)

Field 2: filename (string type)
- Contains a valid filename without extension
- Use lowercase letters with hyphens to separate words
- No special characters except hyphens and underscores
- Example format: add-user-authentication

**Example JSON Response:**

The following is a complete example of the expected JSON format. Notice how:
- Line breaks are represented as \n (backslash followed by n)
- Code blocks with triple backticks are included in the string
- All special characters are properly escaped

{}

**Important Notes:**
- The summary field contains the complete Markdown document as a single JSON string
- Line breaks in markdown must be written as backslash-n (two characters) in the JSON string
- Code blocks in markdown with triple backticks should be included in the summary string
- Ensure all quotes, backslashes, and special characters are properly escaped according to JSON rules
- The JSON must be valid and parseable - test that your response can be parsed as JSON
- When the JSON is parsed, escape sequences will become actual characters (backslash-n becomes a newline)
- Do NOT wrap your JSON response in markdown code fences

## Document Structure Order

The generated document MUST follow this exact order:
1. Level 1 heading with PR Title (format: # PR Title)
2. ## Overview
3. ## Requirements Analysis
   - ### Business Requirements
   - ### Functional Requirements
   - ### User Scenarios
   - ### Impact Analysis
   - ### Change Categories
   - ### Dependencies
4. ## Key Changes
5. ## Files Changed
6. ## Technical Details
7. ## Testing
8. ## Usage Instructions
9. ## Code Changes"#,
        summarize_response_example
    );

    // 使用 LLM 模块的语言增强功能
    get_language_requirement(&base_prompt)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Summarize PR System Prompt Generation Tests ====================

    /// 测试生成PR总结system prompt（无参数）
    ///
    /// ## 测试目的
    /// 验证 `generate_summarize_pr_system_prompt()` 函数能够生成非空的system prompt，并包含所有关键内容。
    ///
    /// ## 测试场景
    /// 1. 调用函数生成prompt
    /// 2. 验证prompt不为空
    /// 3. 验证prompt包含关键内容
    ///
    /// ## 预期结果
    /// - prompt不为空
    /// - 包含 "CRITICAL LANGUAGE REQUIREMENT"
    /// - 包含 "Summary Document Rules"、"Filename Rules"、"Response Format"
    /// - 包含示例关键词（summary, filename, add-user-authentication）
    #[test]
    fn test_generate_summarize_pr_system_prompt_with_no_parameters_returns_prompt() {
        // Arrange: 准备生成prompt

        // Act: 生成PR总结的system prompt
        let prompt = generate_summarize_pr_system_prompt();

        // Assert: 验证返回的prompt不为空且包含关键内容
        assert!(!prompt.is_empty());
        assert!(prompt.contains("CRITICAL LANGUAGE REQUIREMENT"));
        assert!(prompt.contains("Summary Document Rules"));
        assert!(prompt.contains("Filename Rules"));
        assert!(prompt.contains("Response Format"));
        assert!(prompt.contains("summary"));
        assert!(prompt.contains("filename"));
        assert!(prompt.contains("add-user-authentication"));
    }

    /// 测试生成的PR总结prompt包含语言要求
    ///
    /// ## 测试目的
    /// 验证生成的system prompt包含语言要求说明（通过 `get_language_requirement` 添加）。
    ///
    /// ## 测试场景
    /// 1. 生成system prompt
    /// 2. 验证包含语言要求关键词
    ///
    /// ## 预期结果
    /// - prompt包含 "CRITICAL LANGUAGE REQUIREMENT"
    /// - prompt包含 "REMINDER: Language Requirement"
    #[test]
    fn test_generate_summarize_pr_system_prompt_contains_language_requirement_with_prompt_contains_language(
    ) {
        // Arrange: 准备生成prompt
        let prompt = generate_summarize_pr_system_prompt();

        // Act & Assert: 验证包含语言增强内容（通过get_language_requirement添加）
        assert!(prompt.contains("CRITICAL LANGUAGE REQUIREMENT"));
        assert!(prompt.contains("REMINDER: Language Requirement"));
    }

    /// 测试生成的PR总结prompt包含文档结构说明
    ///
    /// ## 测试目的
    /// 验证生成的system prompt包含文档结构说明，包括所有必需的章节。
    ///
    /// ## 测试场景
    /// 1. 生成system prompt
    /// 2. 准备文档结构关键词列表
    /// 3. 验证prompt包含所有关键词
    ///
    /// ## 预期结果
    /// - prompt包含所有文档结构关键词：
    ///   - PR Title, Overview, Requirements Analysis, Key Changes
    ///   - Files Changed, Technical Details, Testing, Usage Instructions
    #[test]
    fn test_generate_summarize_pr_system_prompt_contains_document_structure_with_prompt_contains_structure(
    ) {
        // Arrange: 准备生成prompt和文档结构关键词列表
        let prompt = generate_summarize_pr_system_prompt();
        let structure_keywords = [
            "PR Title",
            "Overview",
            "Requirements Analysis",
            "Key Changes",
            "Files Changed",
            "Technical Details",
            "Testing",
            "Usage Instructions",
        ];

        // Act & Assert: 验证包含文档结构说明
        for keyword in structure_keywords.iter() {
            assert!(prompt.contains(keyword));
        }
    }

    /// 测试多次调用生成函数返回一致的结果
    ///
    /// ## 测试目的
    /// 验证 `generate_summarize_pr_system_prompt()` 函数在多次调用时返回一致的结果（幂等性）。
    ///
    /// ## 测试场景
    /// 1. 多次调用生成函数
    /// 2. 比较多次调用的结果
    ///
    /// ## 预期结果
    /// - 多次调用的结果完全一致
    /// - 函数具有幂等性
    #[test]
    fn test_generate_summarize_pr_system_prompt_consistent_with_multiple_calls_returns_same_result()
    {
        // Arrange: 准备多次调用

        // Act: 多次生成prompt
        let prompt1 = generate_summarize_pr_system_prompt();
        let prompt2 = generate_summarize_pr_system_prompt();

        // Assert: 验证多次调用返回一致的结果
        assert_eq!(prompt1, prompt2);
    }
}
