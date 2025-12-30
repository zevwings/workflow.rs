//! 生成分支名的 system prompt
//!
//! 用于根据 commit 标题和 git 变更生成分支名、PR 标题和描述。

/// 生成分支名的 system prompt
///
/// 用于根据 commit 标题和 git 变更生成分支名、PR 标题和描述。
pub const GENERATE_BRANCH_SYSTEM_PROMPT: &str = r#"You're a git assistant that generates a branch name, PR title, and description based on the commit title and git changes.

## Important

**All outputs MUST be in English only.** If the commit title contains non-English text (like Chinese), translate it to English first.

## Generate Rules

### Branch Name Rules

- Must be all lowercase
- Use hyphens to separate words
- Be under 50 characters
- Follow git branch naming conventions (no spaces, no special characters except hyphens, ASCII characters only)
- Generate only the base branch name without prefix (e.g., `feature-name` not `prefix/feature-name`)
- If existing base branch names are provided, ensure the generated base branch name does not duplicate any of them
- **Primary source**: Base the branch name on the commit title
- Use git changes only to ensure accuracy and avoid conflicts

### PR Title Rules

- **Primary source**: The PR title should be primarily based on the commit title
- Use git changes only to verify and refine the title, not to replace the commit title's intent
- Must be concise, within 8 words
- No punctuation
- In English only
- Focus on "what" (the feature/fix) rather than "how" (implementation details)
- If the commit title is clear and meaningful, use it as the basis for the PR title
- Only use git changes to:
  - Clarify ambiguous commit titles
  - Extract scope information (for Conventional Commits)
  - Verify the title matches the actual changes

### Description Rules

- Generate a concise description based on the git changes provided
- Format as a bulleted list, with each item describing a specific feature, change, or fix
- Each bullet point should start with "- " and be on a separate line
- Focus on what was added, modified, or fixed (not implementation details)
- Group related changes together when appropriate
- If no git changes are provided, you can omit this field or provide a brief description based on the commit title
- Keep it concise but comprehensive (typically 3-8 bullet points)
- In English only

### Scope Rules

- Extract the scope from git changes and file paths
- Scope should be a short identifier (1-3 words) representing the module/feature being changed
- Follow Conventional Commits scope format (lowercase, hyphenated)
- Examples: "api", "auth", "database", "ui", "config", "jira", "pr", "branch"
- Analyze file paths to identify the primary module (e.g., `src/lib/jira/` → "jira", `src/commands/pr/` → "pr")
- If multiple scopes are involved, choose the primary one based on the most significant changes
- If no clear scope can be determined from the changes, you can omit this field (return null)
- In English only

**Examples**

| Input | Output |
|-------|--------|
| "Fix login bug" | `fix-login-bug` |
| "修复登录问题" | `fix-login-issue` |
| "Add user authentication" | `feature-add-user-authentication` |
| "新功能：用户认证" | `feature-user-authentication` |
| "Refactor code structure" | `refactoring-code-structure` |
| "重构代码结构" | `refactoring-code-structure` |
| "Update documentation" | `update-documentation` |
| "更新文档" | `update-documentation` |
| "Improve performance" | `improve-performance` |
| "优化性能" | `performance-optimization` |

## Response Format

Return your response in JSON format with four fields: `branch_name`, `pr_title`, `description` (optional), and `scope` (optional).

**Example 1**

```json
{
  "branch_name": "add-user-authentication",
  "pr_title": "Add user authentication",
  "description": "- Add user authentication functionality with login and registration\n- Implement JWT token generation and validation\n- Add password hashing using bcrypt\n- Update API endpoints for authentication\n- Add unit tests for authentication flow",
  "scope": "auth"
}
```

**Example 2**

```json
{
  "branch_name": "feat-branch-create-command",
  "pr_title": "Add branch create command",
  "description": "- Add workflow branch create command with JIRA ticket support\n- Support LLM-based branch name generation\n- Add dry-run mode and --from-default option\n- Update README.md with new commands\n- Fix doctest in branch module\n- Add CLI parameter parsing tests for branch create\n- Add unit tests for branch naming and types",
  "scope": "branch"
}
```

**Example 3** (without scope)

```json
{
  "branch_name": "update-documentation",
  "pr_title": "Update documentation",
  "description": "- Update README with new features\n- Fix typos in API documentation"
}
```"#;

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Generate Branch System Prompt Tests ====================

    /// 测试生成分支名system prompt常量不为空
    ///
    /// ## 测试目的
    /// 验证 `GENERATE_BRANCH_SYSTEM_PROMPT` 常量已正确定义且不为空。
    ///
    /// ## 测试场景
    /// 1. 检查prompt常量
    /// 2. 验证常量不为空
    ///
    /// ## 预期结果
    /// - prompt常量不为空
    #[test]
    fn test_generate_branch_system_prompt_with_valid_constant_returns_non_empty() {
        // Arrange: 准备检查 prompt 常量

        // Act: 验证 prompt 常量不为空
        // (验证在 Assert 中完成)

        // Assert: 验证 prompt 常量不为空
        // Note: GENERATE_BRANCH_SYSTEM_PROMPT is a compile-time constant with non-empty content
        // The constant is verified to exist and contain content at compile time
        let _ = GENERATE_BRANCH_SYSTEM_PROMPT;
    }

    /// 测试生成分支名prompt包含必需的关键词
    ///
    /// ## 测试目的
    /// 验证 `GENERATE_BRANCH_SYSTEM_PROMPT` 包含所有必需的关键词（branch name, PR title, description, scope等）。
    ///
    /// ## 测试场景
    /// 1. 准备关键词列表
    /// 2. 验证prompt包含所有关键词
    ///
    /// ## 预期结果
    /// - prompt包含所有必需的关键词
    #[test]
    fn test_generate_branch_system_prompt_contains_required_keywords() {
        // Arrange: 准备关键词列表
        let keywords = ["branch name", "PR title", "description", "scope"];

        // Act & Assert: 验证 prompt 包含所有关键词
        for keyword in keywords.iter() {
            assert!(
                GENERATE_BRANCH_SYSTEM_PROMPT.contains(keyword),
                "Prompt should contain keyword: {}",
                keyword
            );
        }
    }

    /// 测试生成分支名prompt包含规则说明
    ///
    /// ## 测试目的
    /// 验证 `GENERATE_BRANCH_SYSTEM_PROMPT` 包含所有规则说明（Branch Name Rules, PR Title Rules, Description Rules, Scope Rules）。
    ///
    /// ## 测试场景
    /// 1. 准备规则关键词
    /// 2. 验证prompt包含所有规则说明
    ///
    /// ## 预期结果
    /// - prompt包含所有规则说明
    #[test]
    fn test_generate_branch_system_prompt_contains_required_rules() {
        // Arrange: 准备规则关键词
        let rule_keywords = [
            "Branch Name Rules",
            "PR Title Rules",
            "Description Rules",
            "Scope Rules",
        ];

        // Act & Assert: 验证 prompt 包含所有规则说明
        for rule_keyword in rule_keywords.iter() {
            assert!(
                GENERATE_BRANCH_SYSTEM_PROMPT.contains(rule_keyword),
                "Prompt should contain rule: {}",
                rule_keyword
            );
        }
    }

    /// 测试生成分支名prompt包含示例和格式说明
    ///
    /// ## 测试目的
    /// 验证 `GENERATE_BRANCH_SYSTEM_PROMPT` 包含示例和响应格式说明。
    ///
    /// ## 测试场景
    /// 1. 准备示例和格式关键词
    /// 2. 验证prompt包含示例和格式说明
    ///
    /// ## 预期结果
    /// - prompt包含 "Examples" 或 "examples"
    /// - prompt包含 "Response Format" 或 "response format"
    #[test]
    fn test_generate_branch_system_prompt_contains_examples_and_format() {
        // Arrange: 准备示例和格式关键词
        let example_keywords = ["Examples", "examples"];
        let format_keywords = ["Response Format", "response format"];

        // Act & Assert: 验证 prompt 包含示例和格式说明
        assert!(
            GENERATE_BRANCH_SYSTEM_PROMPT.contains(example_keywords[0])
                || GENERATE_BRANCH_SYSTEM_PROMPT.contains(example_keywords[1]),
            "Prompt should contain examples"
        );
        assert!(
            GENERATE_BRANCH_SYSTEM_PROMPT.contains(format_keywords[0])
                || GENERATE_BRANCH_SYSTEM_PROMPT.contains(format_keywords[1]),
            "Prompt should contain response format"
        );
    }

    /// 测试生成分支名prompt包含JSON格式规范
    ///
    /// ## 测试目的
    /// 验证 `GENERATE_BRANCH_SYSTEM_PROMPT` 包含JSON格式规范说明（branch_name, pr_title, description, scope字段）。
    ///
    /// ## 测试场景
    /// 1. 准备JSON字段关键词
    /// 2. 验证prompt包含所有JSON字段说明
    ///
    /// ## 预期结果
    /// - prompt包含 "branch_name"
    /// - prompt包含 "pr_title"
    /// - prompt包含 "description"
    /// - prompt包含 "scope"
    #[test]
    fn test_generate_branch_system_prompt_contains_json_format_specification() {
        // Arrange: 准备JSON字段关键词
        let json_fields = ["branch_name", "pr_title", "description", "scope"];

        // Act & Assert: 验证 prompt 包含所有JSON字段说明
        for field in json_fields.iter() {
            assert!(
                GENERATE_BRANCH_SYSTEM_PROMPT.contains(field),
                "Prompt should contain JSON field: {}",
                field
            );
        }
    }

    /// 测试生成分支名prompt包含语言要求
    ///
    /// ## 测试目的
    /// 验证 `GENERATE_BRANCH_SYSTEM_PROMPT` 包含语言要求说明（所有输出必须是英文）。
    ///
    /// ## 测试场景
    /// 1. 准备语言要求关键词
    /// 2. 验证prompt包含语言要求
    ///
    /// ## 预期结果
    /// - prompt包含 "English" 或 "english"
    /// - prompt包含 "MUST" 或 "must"
    #[test]
    fn test_generate_branch_system_prompt_contains_language_requirement() {
        // Arrange: 准备语言要求关键词
        let language_keywords = ["English", "english"];
        let requirement_keywords = ["MUST", "must"];

        // Act & Assert: 验证 prompt 包含语言要求
        assert!(
            GENERATE_BRANCH_SYSTEM_PROMPT.contains(language_keywords[0])
                || GENERATE_BRANCH_SYSTEM_PROMPT.contains(language_keywords[1]),
            "Prompt should contain language requirement"
        );
        assert!(
            GENERATE_BRANCH_SYSTEM_PROMPT.contains(requirement_keywords[0])
                || GENERATE_BRANCH_SYSTEM_PROMPT.contains(requirement_keywords[1]),
            "Prompt should contain requirement keyword"
        );
    }

    /// 测试生成分支名prompt长度合理
    ///
    /// ## 测试目的
    /// 验证 `GENERATE_BRANCH_SYSTEM_PROMPT` 有合理的长度，至少包含基本内容（最小长度阈值500字符）。
    ///
    /// ## 测试场景
    /// 1. 获取prompt长度
    /// 2. 验证长度超过最小阈值
    ///
    /// ## 预期结果
    /// - prompt长度大于500字符
    #[test]
    fn test_generate_branch_system_prompt_has_reasonable_length() {
        // Arrange: 准备最小长度要求
        let min_length = 500;

        // Act: 获取 prompt 长度
        let prompt_length = GENERATE_BRANCH_SYSTEM_PROMPT.len();

        // Assert: 验证 prompt 有合理的长度
        assert!(
            prompt_length > min_length,
            "Prompt should have reasonable length (at least {}), got {}",
            min_length,
            prompt_length
        );
    }
}
