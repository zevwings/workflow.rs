//! Base/Prompt Generate Branch 模块测试
//!
//! 测试生成分支名的 system prompt。

use workflow::base::prompt::GENERATE_BRANCH_SYSTEM_PROMPT;

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
    assert!(!GENERATE_BRANCH_SYSTEM_PROMPT.is_empty());
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
/// 1. 准备规则关键词列表
/// 2. 验证prompt包含所有规则说明
///
/// ## 预期结果
/// - prompt包含所有规则说明
#[test]
fn test_generate_branch_system_prompt_contains_required_rules() {
    // Arrange: 准备规则关键词列表
    let rules = [
        "Branch Name Rules",
        "PR Title Rules",
        "Description Rules",
        "Scope Rules",
    ];

    // Act & Assert: 验证 prompt 包含所有规则说明
    for rule in rules.iter() {
        assert!(
            GENERATE_BRANCH_SYSTEM_PROMPT.contains(rule),
            "Prompt should contain rule: {}",
            rule
        );
    }
}

/// 测试生成分支名prompt包含示例和格式说明
///
/// ## 测试目的
/// 验证 `GENERATE_BRANCH_SYSTEM_PROMPT` 包含使用示例和响应格式说明。
///
/// ## 测试场景
/// 1. 准备示例和格式关键词
/// 2. 验证prompt包含这些内容
///
/// ## 预期结果
/// - prompt包含 "Examples" 和 "Response Format"
#[test]
fn test_generate_branch_system_prompt_contains_examples_and_format() {
    // Arrange: 准备示例和格式关键词
    let required_content = ["Examples", "Response Format"];

    // Act & Assert: 验证 prompt 包含示例和格式说明
    for content in required_content.iter() {
        assert!(
            GENERATE_BRANCH_SYSTEM_PROMPT.contains(content),
            "Prompt should contain: {}",
            content
        );
    }
}

/// 测试生成分支名prompt包含JSON格式说明
///
/// ## 测试目的
/// 验证 `GENERATE_BRANCH_SYSTEM_PROMPT` 包含JSON响应格式的说明。
///
/// ## 测试场景
/// 1. 准备JSON格式相关关键词
/// 2. 验证prompt包含JSON格式说明
///
/// ## 预期结果
/// - prompt包含 "JSON"、"branch_name"、"pr_title" 等关键词
#[test]
fn test_generate_branch_system_prompt_contains_json_format_specification() {
    // Arrange: 准备 JSON 格式相关关键词
    let json_keywords = ["JSON", "branch_name", "pr_title"];

    // Act & Assert: 验证 prompt 包含 JSON 格式说明
    for keyword in json_keywords.iter() {
        assert!(
            GENERATE_BRANCH_SYSTEM_PROMPT.contains(keyword),
            "Prompt should contain JSON format keyword: {}",
            keyword
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
/// - prompt包含 "English" 和 "All outputs MUST be in English"
#[test]
fn test_generate_branch_system_prompt_contains_language_requirement() {
    // Arrange: 准备语言要求关键词
    let language_requirements = ["English", "All outputs MUST be in English"];

    // Act & Assert: 验证 prompt 包含语言要求
    for requirement in language_requirements.iter() {
        assert!(
            GENERATE_BRANCH_SYSTEM_PROMPT.contains(requirement),
            "Prompt should contain language requirement: {}",
            requirement
        );
    }
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
