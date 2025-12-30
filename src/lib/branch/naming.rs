//! Branch naming module
//!
//! Provides branch name generation from various sources:
//! - JIRA tickets (with template system, LLM support, and fallback)
//! - Titles/text
//! - Templates (when template system is available)

use crate::branch::llm::BranchLLM;
use crate::git::GitBranch;
use crate::pr::llm::CreateGenerator;
use crate::repo::config::RepoConfig;
use crate::template::{BranchTemplateVars, TemplateConfig, TemplateEngine};
use crate::{log_info, log_success, log_warning};
use color_eyre::Result;

/// Branch naming service
///
/// Provides methods for generating branch names from various sources.
/// Uses a strategy pattern: template system → LLM → simple fallback.
pub struct BranchNaming;

impl BranchNaming {
    /// Generate branch name from JIRA ticket (with template system, LLM, or fallback)
    ///
    /// Uses template system first, then tries LLM, finally falls back to simple method.
    ///
    /// # Arguments
    ///
    /// * `ticket_id` - JIRA ticket ID (e.g., "PROJ-123")
    /// * `summary` - JIRA ticket summary
    /// * `jira_type` - Optional JIRA ticket type (e.g., "Feature", "Bug")
    /// * `use_prefix_format` - Whether to use `prefix/ticket-slug` format (true) or `ticket--slug` format (false)
    ///
    /// # Returns
    ///
    /// Returns generated branch name (prefixes are handled automatically by the template system)
    pub fn from_jira_ticket(
        ticket_id: &str,
        summary: &str,
        jira_type: Option<&str>,
        use_prefix_format: bool,
    ) -> Result<String> {
        // Try template system first
        match TemplateConfig::load_branch_template(jira_type) {
            Ok(template_str) => {
                // Prepare template variables
                let summary_slug = Self::slugify(summary);
                let vars = BranchTemplateVars {
                    jira_key: Some(ticket_id.to_string()),
                    jira_summary: Some(summary.to_string()),
                    summary_slug: Some(summary_slug),
                    jira_type: jira_type.map(|s| s.to_string()),
                };

                // Render template
                let engine = TemplateEngine::new();
                match engine.render_string(&template_str, &vars) {
                    Ok(rendered) => {
                        log_success!("Generated branch name using template: {}", rendered);
                        // Apply format conversion if needed
                        let formatted = if use_prefix_format {
                            // Template already includes prefix, but still check for repo prefix
                            rendered
                        } else {
                            // Convert prefix/ticket-slug to ticket--slug format
                            Self::convert_to_double_dash_format(&rendered, ticket_id)
                        };
                        // Apply repository prefix if needed
                        Ok(Self::apply_repo_prefix_if_needed(formatted))
                    }
                    Err(e) => {
                        log_warning!(
                            "Failed to render branch template: {}, trying LLM fallback",
                            e
                        );
                        // Fall through to LLM
                        Self::try_llm_generation(ticket_id, summary, use_prefix_format)
                    }
                }
            }
            Err(_) => {
                // Template system not available, try LLM
                Self::try_llm_generation(ticket_id, summary, use_prefix_format)
            }
        }
    }

    /// Generate branch name using template and branch type
    ///
    /// Generates a branch name using template system based on branch type.
    /// This is the new recommended method for branch name generation.
    ///
    /// # Arguments
    ///
    /// * `branch_type` - Branch type (feature/bugfix/refactoring/hotfix/chore)
    /// * `branch_name_slug` - Branch name slug (already sanitized)
    /// * `jira_ticket` - Optional JIRA ticket ID
    ///
    /// # Returns
    ///
    /// Returns generated branch name (prefixes are handled automatically by the template system)
    pub fn from_type_and_slug(
        branch_type: &str,
        branch_name_slug: &str,
        jira_ticket: Option<&str>,
    ) -> Result<String> {
        // If no JIRA ticket, use simple format: {type}/{slug}
        if jira_ticket.is_none() {
            let branch_name = format!("{}/{}", branch_type, branch_name_slug);
            return Ok(Self::apply_repo_prefix_if_needed(branch_name));
        }

        // If JIRA ticket exists, use template system
        // Load template for the branch type
        let template_str = TemplateConfig::load_branch_template_by_type(Some(branch_type))?;

        // Prepare template variables
        let vars = BranchTemplateVars {
            jira_key: jira_ticket.map(|s| s.to_string()),
            jira_summary: None,
            summary_slug: Some(branch_name_slug.to_string()),
            jira_type: None,
        };

        // Render template
        let engine = TemplateEngine::new();
        let rendered = engine.render_string(&template_str, &vars)?;

        // Apply repository prefix if needed
        Ok(Self::apply_repo_prefix_if_needed(rendered))
    }

    /// Generate branch name from title
    ///
    /// Generates a branch name from a title string, optionally with JIRA ticket prefix.
    ///
    /// # Arguments
    ///
    /// * `jira_ticket` - Optional JIRA ticket ID
    /// * `title` - Title string
    ///
    /// # Returns
    ///
    /// Returns generated branch name (prefixes are handled automatically)
    pub fn from_title(jira_ticket: Option<&str>, title: &str) -> Result<String> {
        let mut branch_name = String::new();

        // If JIRA ticket exists, add as prefix
        if let Some(ticket) = jira_ticket {
            branch_name.push_str(ticket);
            branch_name.push_str("--");
        }

        // Clean title as branch name
        let cleaned_title = Self::sanitize(title);
        branch_name.push_str(&cleaned_title);

        // Apply repository prefix if needed
        Ok(Self::apply_repo_prefix_if_needed(branch_name))
    }

    /// Sanitize string to branch name format
    ///
    /// Converts string to branch name format:
    /// - Replace special characters with hyphens
    /// - Remove duplicate hyphens
    /// - Only keep ASCII alphanumeric characters, filter out non-ASCII characters (like Chinese)
    pub fn sanitize(s: &str) -> String {
        let mut result = String::new();
        let mut last_was_dash = false;

        for c in s.chars() {
            // Only keep ASCII alphanumeric characters, completely ignore non-ASCII characters (like Chinese)
            if c.is_ascii_alphanumeric() {
                result.push(c.to_ascii_lowercase());
                last_was_dash = false;
            } else if c.is_ascii() {
                // For ASCII non-alphanumeric characters (like spaces, punctuation), convert to hyphens
                if !last_was_dash {
                    result.push('-');
                    last_was_dash = true;
                }
            }
            // Completely ignore non-ASCII characters (like Chinese), don't add hyphens
        }

        result.trim_matches('-').to_string()
    }

    /// Convert summary to slug format (for fallback method)
    ///
    /// Similar to `sanitize`, but preserves more characters (including underscores)
    pub fn slugify(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else if c.is_whitespace() {
                    '-'
                } else {
                    '\0'
                }
            })
            .filter(|c| *c != '\0')
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
            .chars()
            .take(50) // Limit length
            .collect::<String>()
            .trim_end_matches('-')
            .to_string()
    }

    /// Try LLM generation (fallback method)
    fn try_llm_generation(
        ticket_id: &str,
        summary: &str,
        use_prefix_format: bool,
    ) -> Result<String> {
        let exists_branches = GitBranch::get_all_branches(true).ok();
        let git_diff = None;

        match CreateGenerator::generate(summary, exists_branches, git_diff) {
            Ok(content) => {
                log_success!("Generated branch name using LLM: {}", content.branch_name);
                let base_name = content.branch_name;
                let formatted =
                    Self::format_branch_name_with_ticket(use_prefix_format, ticket_id, &base_name);
                // Apply repository prefix if needed
                Ok(Self::apply_repo_prefix_if_needed(formatted))
            }
            Err(e) => {
                log_warning!(
                    "Failed to generate branch name using LLM: {}, falling back to simple method",
                    e
                );
                // Fallback to simple method
                let slug = Self::slugify(summary);
                let formatted =
                    Self::format_branch_name_simple(use_prefix_format, ticket_id, &slug);
                // Apply repository prefix if needed
                Ok(Self::apply_repo_prefix_if_needed(formatted))
            }
        }
    }

    /// Convert prefix/ticket-slug format to ticket--slug format
    fn convert_to_double_dash_format(branch_name: &str, ticket_id: &str) -> String {
        // Remove prefix/ if exists
        let without_prefix = if let Some(slash_pos) = branch_name.find('/') {
            &branch_name[slash_pos + 1..]
        } else {
            branch_name
        };

        // Replace first hyphen after ticket_id with double dash
        let ticket_prefix = format!("{}-", ticket_id);
        if without_prefix.starts_with(&ticket_prefix) {
            format!("{}--{}", ticket_id, &without_prefix[ticket_prefix.len()..])
        } else {
            without_prefix.to_string()
        }
    }

    /// Format branch name with ticket (LLM generated)
    fn format_branch_name_with_ticket(
        use_prefix_format: bool,
        ticket_id: &str,
        base_name: &str,
    ) -> String {
        if use_prefix_format {
            let prefix = "feature"; // Default prefix, can be enhanced with issue type
            format!("{}/{}-{}", prefix, ticket_id, base_name)
        } else {
            format!("{}--{}", ticket_id, base_name)
        }
    }

    /// Format branch name (simple method)
    fn format_branch_name_simple(use_prefix_format: bool, ticket_id: &str, slug: &str) -> String {
        if use_prefix_format {
            let prefix = "feature";
            if slug.is_empty() {
                format!("{}/{}", prefix, ticket_id)
            } else {
                format!("{}/{}-{}", prefix, ticket_id, slug)
            }
        } else if slug.is_empty() {
            ticket_id.to_string()
        } else {
            format!("{}--{}", ticket_id, slug)
        }
    }

    /// Apply repository prefix to branch name if configured and not already present
    ///
    /// This method ensures that all branch names respect the repository-level
    /// branch prefix configuration, while avoiding duplicate prefixes.
    ///
    /// # Arguments
    ///
    /// * `branch_name` - The branch name to potentially prefix
    ///
    /// # Returns
    ///
    /// Returns the branch name with repository prefix applied if:
    /// - Repository prefix is configured
    /// - Branch name doesn't already start with the prefix
    fn apply_repo_prefix_if_needed(branch_name: String) -> String {
        if let Some(prefix) = RepoConfig::get_branch_prefix() {
            let trimmed = prefix.trim();
            if !trimmed.is_empty() {
                // Check if branch name already starts with the prefix
                let prefix_with_slash = format!("{}/", trimmed);
                if !branch_name.starts_with(&prefix_with_slash) {
                    return format!("{}/{}", trimmed, branch_name);
                }
            }
        }
        branch_name
    }

    /// 清理并翻译分支名称（处理非英文输入）
    ///
    /// 将用户输入转换为有效的分支名称 slug。
    /// 如果输入包含非英文字符，会先使用 LLM 翻译为英文。
    ///
    /// # 参数
    ///
    /// * `input` - 用户输入的分支名称
    ///
    /// # 返回
    ///
    /// 返回清理后的分支名称 slug
    ///
    /// # 错误
    ///
    /// 如果清理后分支名称为空，返回错误
    pub fn sanitize_and_translate_branch_name(input: &str) -> Result<String> {
        // Check if input contains non-ASCII characters (likely non-English)
        let has_non_ascii = !input.is_ascii();

        let text_to_sanitize = if has_non_ascii {
            // Use LLM to translate non-English input to English
            log_info!("Detected non-English input, translating to English...");
            match BranchLLM::translate_to_english(input) {
                Ok(translated) => {
                    log_success!("Translated to English: {}", translated);
                    translated
                }
                Err(e) => {
                    log_warning!(
                        "Failed to translate using LLM: {}. Using original input.",
                        e
                    );
                    // Fallback: use original input and let sanitize handle it
                    input.to_string()
                }
            }
        } else {
            // Already in English (or ASCII only), use as-is
            input.to_string()
        };

        // Sanitize the text (now should be in English)
        let sanitized = Self::sanitize(&text_to_sanitize);

        // Check if sanitization removed too much (likely because translation failed and input was non-ASCII)
        if sanitized.is_empty() {
            // If sanitization removed everything, try slugify
            let slug = Self::slugify(&text_to_sanitize);
            if slug.is_empty() {
                color_eyre::eyre::bail!(
                    "Branch name cannot be empty after sanitization. The input '{}' contains non-English characters that were removed. Please provide an English title or a JIRA ticket ID for better branch name generation.",
                    input
                );
            }
            // If slug is too short (less than 3 characters), it's likely incomplete
            if slug.len() < 3 && has_non_ascii {
                color_eyre::eyre::bail!(
                    "Branch name '{}' is too short after sanitization. The input '{}' contains non-English characters that were removed. Please provide an English title or a JIRA ticket ID for better branch name generation.",
                    slug,
                    input
                );
            }
            Ok(slug)
        } else if sanitized.len() < 3 && has_non_ascii {
            // If sanitized result is too short and original input had non-ASCII, warn user
            log_warning!(
                "Generated branch name '{}' is very short. This may be because the original input '{}' contained non-English characters that were removed. Consider providing an English title or a JIRA ticket ID for better results.",
                sanitized,
                input
            );
            Ok(sanitized)
        } else {
            Ok(sanitized)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    /// 测试基本的字符串清理功能
    ///
    /// ## 测试目的
    /// 验证 BranchNaming::sanitize() 能够将输入字符串转换为有效的分支名称格式。
    ///
    /// ## 测试场景
    /// 1. 测试空格转换为连字符
    /// 2. 测试特殊字符（@、#）转换为连字符
    ///
    /// ## 预期结果
    /// - "Hello World" 被清理为 "hello-world"
    /// - "test@branch#123" 被清理为 "test-branch-123"
    #[test]
    fn test_sanitize_basic() {
        // Arrange & Act & Assert: 验证基本的字符串清理功能
        assert_eq!(BranchNaming::sanitize("Hello World"), "hello-world");
        assert_eq!(BranchNaming::sanitize("test@branch#123"), "test-branch-123");
    }

    /// 测试基本的 slug 化功能
    ///
    /// ## 测试目的
    /// 验证 BranchNaming::slugify() 能够将简单的字符串转换为 URL 友好的 slug 格式。
    ///
    /// ## 测试场景
    /// 测试带空格的字符串转换为连字符分隔的小写字符串
    ///
    /// ## 预期结果
    /// - "Hello World" 被转换为 "hello-world"
    #[test]
    fn test_slugify_basic() {
        // Arrange & Act & Assert: 验证基本的 slugify 功能
        assert_eq!(BranchNaming::slugify("Hello World"), "hello-world");
    }

    /// 测试 slugify 保留下划线
    ///
    /// ## 测试目的
    /// 验证 BranchNaming::slugify() 在转换过程中保留下划线字符。
    ///
    /// ## 测试场景
    /// 1. 测试单个下划线的保留
    /// 2. 测试多个下划线的保留
    ///
    /// ## 预期结果
    /// - "test_branch" 保留为 "test_branch"
    /// - "test_branch_name" 保留为 "test_branch_name"
    #[test]
    fn test_slugify_preserves_underscores() {
        // Arrange & Act & Assert: 验证 slugify 保留下划线
        assert_eq!(BranchNaming::slugify("test_branch"), "test_branch");
        assert_eq!(
            BranchNaming::slugify("test_branch_name"),
            "test_branch_name"
        );
    }

    /// 测试 slugify 长度限制
    ///
    /// ## 测试目的
    /// 验证 BranchNaming::slugify() 能够限制输出字符串的长度不超过 50 个字符。
    ///
    /// ## 测试场景
    /// 测试 100 个字符的长字符串被截断为 50 个字符以内
    ///
    /// ## 预期结果
    /// - 输入 100 个 'a' 字符
    /// - 输出长度不超过 50 个字符
    #[test]
    fn test_slugify_length_limit() {
        // Arrange: 准备 100 个字符的长字符串
        let long_text = "a".repeat(100);

        // Act: 执行 slugify 操作
        let result = BranchNaming::slugify(&long_text);

        // Assert: 验证结果长度不超过 50 个字符
        assert!(result.len() <= 50);
    }

    /// 测试 slugify 过滤空段
    ///
    /// ## 测试目的
    /// 验证 BranchNaming::slugify() 能够移除连续的连字符和多余的空格。
    ///
    /// ## 测试场景
    /// 1. 测试连续的连字符被合并为单个连字符
    /// 2. 测试多个空格被转换为单个连字符
    ///
    /// ## 预期结果
    /// - "test---branch" 被转换为 "test-branch"
    /// - "test   branch" 被转换为 "test-branch"
    #[test]
    fn test_slugify_filters_empty_segments() {
        // Arrange & Act & Assert: 验证 slugify 过滤空段
        assert_eq!(BranchNaming::slugify("test---branch"), "test-branch");
        assert_eq!(BranchNaming::slugify("test   branch"), "test-branch");
    }

    // ==================== from_type_and_slug 函数测试 ====================

    /// 测试从类型和 slug 生成分支名称（无工单）
    ///
    /// ## 测试目的
    /// 验证 BranchNaming::from_type_and_slug() 能够在没有工单号的情况下生成正确的分支名称。
    ///
    /// ## 测试场景
    /// 使用分支类型 "feature" 和 slug "my-branch"，不提供工单号
    ///
    /// ## 预期结果
    /// - 生成的分支名称以 "feature/my-branch" 结尾
    /// - 可能包含仓库前缀（如果已配置）
    #[test]
    fn test_from_type_and_slug_without_ticket() {
        // Arrange: 准备分支类型和 slug
        // Act: 生成分支名称（不提供工单号）
        let result = BranchNaming::from_type_and_slug("feature", "my-branch", None).unwrap();

        // Assert: 验证结果以预期格式结尾（可能包含仓库前缀）
        assert!(
            result.ends_with("feature/my-branch"),
            "Expected result to end with 'feature/my-branch', got: {}",
            result
        );
    }

    /// 测试从类型和 slug 生成分支名称（带工单）
    ///
    /// ## 测试目的
    /// 验证 BranchNaming::from_type_and_slug() 能够在提供工单号的情况下生成正确的分支名称。
    ///
    /// ## 测试场景
    /// 使用分支类型 "feature"、slug "my-branch" 和工单号 "PROJ-123"
    ///
    /// ## 预期结果
    /// - 函数调用成功（返回 Ok）
    /// - 生成的分支名称包含类型或 slug 信息
    /// - 注意：具体格式取决于模板系统的配置
    #[test]
    fn test_from_type_and_slug_with_ticket() {
        // Arrange: 准备分支类型、slug 和工单号
        // Act: 生成分支名称（提供工单号）
        let result = BranchNaming::from_type_and_slug("feature", "my-branch", Some("PROJ-123"));

        // Assert: 验证函数调用成功
        assert!(result.is_ok());

        // Assert: 验证结果包含类型或 slug（具体格式取决于模板系统）
        let branch_name = result.unwrap();
        assert!(branch_name.contains("feature") || branch_name.contains("my-branch"));
    }

    // ==================== Slugify Tests (from naming_utils.rs) ====================

    /// 测试 slugify 功能（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 BranchNaming::slugify() 能够正确处理各种输入场景。
    ///
    /// ## 测试场景
    /// 测试以下场景：
    /// - 基本输入（空格转换为连字符）
    /// - 保留下划线
    /// - 移除特殊字符
    /// - 空字符串处理
    /// - 规范化空白字符
    ///
    /// ## 预期结果
    /// - 所有输入都被正确处理并返回预期的 slugified 字符串
    #[rstest]
    #[case("Hello World", "hello-world")] // 基本输入：空格转换为连字符
    #[case("test branch", "test-branch")] // 基本输入
    #[case("Test Branch Name", "test-branch-name")] // 多个单词
    #[case("test_branch", "test_branch")] // 保留下划线
    #[case("test_branch_name", "test_branch_name")] // 多个下划线
    #[case("test@branch#123", "testbranch123")] // 移除特殊字符
    #[case("test.branch", "testbranch")] // 移除点号
    #[case("", "")] // 空字符串
    #[case("  test  branch  ", "test-branch")] // 规范化前后空格
    #[case("test   branch", "test-branch")] // 规范化多个空格
    fn test_slugify_with_various_inputs_returns_slugified_string(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        // Arrange: 准备输入字符串（通过参数传入）

        // Act: 调用 slugify 方法
        let result = BranchNaming::slugify(input);

        // Assert: 验证返回正确的 slugified 字符串
        assert_eq!(result, expected, "Failed to slugify '{}'", input);
    }

    // ==================== Sanitize Tests (from naming_utils.rs) ====================

    /// 测试 sanitize 功能（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 BranchNaming::sanitize() 能够正确处理各种输入场景。
    ///
    /// ## 测试场景
    /// 测试以下场景：
    /// - 基本输入（空格转换为连字符）
    /// - 转换特殊字符为连字符
    /// - 移除非 ASCII 字符
    /// - 移除重复连字符
    /// - 修剪前导和尾随连字符
    /// - 空字符串和只包含特殊字符的输入
    ///
    /// ## 预期结果
    /// - 所有输入都被正确处理并返回预期的 sanitized 字符串
    #[rstest]
    #[case("Hello World", "hello-world")] // 基本输入：空格转换为连字符
    #[case("test-branch", "test-branch")] // 已包含连字符
    #[case("Test Branch", "test-branch")] // 多个单词
    #[case("test@branch#123", "test-branch-123")] // 转换特殊字符为连字符
    #[case("test.branch", "test-branch")] // 转换点号为连字符
    #[case("test_branch", "test-branch")] // 转换下划线为连字符
    #[case("测试分支", "")] // 移除非ASCII字符（纯中文）
    #[case("test中文branch", "testbranch")] // 移除非ASCII字符（混合）
    #[case("test 中文 branch", "test-branch")] // 移除非ASCII字符（带空格）
    #[case("Hello 世界", "hello")] // 移除非ASCII字符
    #[case("test---branch", "test-branch")] // 移除重复连字符
    #[case("test   branch", "test-branch")] // 多个空格转换为单个连字符
    #[case("-test-branch-", "test-branch")] // 修剪前后连字符
    #[case("--test--", "test")] // 修剪多个前后连字符
    #[case("", "")] // 空字符串
    #[case("@#$%", "")] // 只包含特殊字符
    #[case("---", "")] // 只包含连字符
    fn test_sanitize_with_various_inputs_returns_sanitized_string(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        // Arrange: 准备输入字符串（通过参数传入）

        // Act: 调用 sanitize 方法
        let result = BranchNaming::sanitize(input);

        // Assert: 验证返回正确的 sanitized 字符串
        assert_eq!(result, expected, "Failed to sanitize '{}'", input);
    }

    // ==================== Slugify Boundary Tests (from naming_advanced.rs) ====================

    /// 测试slugify方法对长字符串的长度限制
    ///
    /// ## 测试目的
    /// 验证 `BranchNaming::slugify()` 方法能够正确处理超过长度限制的长字符串，结果不超过50个字符。
    ///
    /// ## 测试场景
    /// 1. 准备超过长度限制的长字符串（100个字符）
    /// 2. 调用slugify方法
    /// 3. 验证结果长度不超过50个字符
    ///
    /// ## 预期结果
    /// - 结果长度不超过50个字符
    #[test]
    fn test_slugify_with_long_string_enforces_length_limit() {
        // Arrange: 准备超过长度限制的长字符串（100个字符）
        let long_string = "a".repeat(100);

        // Act: 调用 slugify 方法
        let result = BranchNaming::slugify(&long_string);

        // Assert: 验证结果不超过50个字符
        assert!(result.len() <= 50);
    }

    /// 测试slugify方法处理边界情况
    ///
    /// ## 测试目的
    /// 验证 `BranchNaming::slugify()` 方法能够正确处理各种边界情况（空字符串、空格、连字符、单个字符等）。
    ///
    /// ## 测试场景
    /// 1. 测试空字符串
    /// 2. 测试只有空格的字符串
    /// 3. 测试只有连字符的字符串
    /// 4. 测试单个小写字母
    /// 5. 测试单个大写字母
    ///
    /// ## 预期结果
    /// - 空字符串和空格返回空字符串
    /// - 连字符返回空字符串
    /// - 单个字母返回小写字母
    #[test]
    fn test_slugify_with_edge_cases_handles_correctly() {
        // Arrange: 准备边界情况输入
        let empty_input = "";
        let whitespace_input = "   ";
        let hyphens_input = "---";
        let single_lowercase = "a";
        let single_uppercase = "A";

        // Act: 调用 slugify 方法
        let result_empty = BranchNaming::slugify(empty_input);
        let result_whitespace = BranchNaming::slugify(whitespace_input);
        let result_hyphens = BranchNaming::slugify(hyphens_input);
        let result_lowercase = BranchNaming::slugify(single_lowercase);
        let result_uppercase = BranchNaming::slugify(single_uppercase);

        // Assert: 验证边界情况处理正确
        assert_eq!(result_empty, "");
        assert_eq!(result_whitespace, "");
        assert_eq!(result_hyphens, "");
        assert_eq!(result_lowercase, "a");
        assert_eq!(result_uppercase, "a");
    }

    /// 测试slugify方法处理Unicode字符
    ///
    /// ## 测试目的
    /// 验证 `BranchNaming::slugify()` 方法能够正确处理包含Unicode字符的输入（保留ASCII部分）。
    ///
    /// ## 测试场景
    /// 1. 测试包含Unicode字符的输入（café, naïve）
    /// 2. 调用slugify方法
    /// 3. 验证Unicode字符处理正确
    ///
    /// ## 预期结果
    /// - Unicode字符被移除或转换
    /// - ASCII部分被保留
    #[test]
    fn test_slugify_with_unicode_characters_handles_correctly() {
        // Arrange: 准备包含 Unicode 字符的输入
        let input1 = "café";
        let input2 = "naïve";

        // Act: 调用 slugify 方法
        let result1 = BranchNaming::slugify(input1);
        let result2 = BranchNaming::slugify(input2);

        // Assert: 验证 Unicode 字符处理正确（保留 ASCII 部分）
        assert!(result1.contains("caf"));
        assert!(result2.contains("na"));
    }

    /// 测试slugify方法保留数字
    ///
    /// ## 测试目的
    /// 验证 `BranchNaming::slugify()` 方法能够保留输入中的数字。
    ///
    /// ## 测试场景
    /// 1. 测试包含数字的输入（test123, 123test, test-123-branch）
    /// 2. 调用slugify方法
    /// 3. 验证数字被保留
    ///
    /// ## 预期结果
    /// - 数字被保留在结果中
    /// - 格式正确（小写、连字符分隔）
    #[test]
    fn test_slugify_with_numbers_preserves_numbers() {
        // Arrange: 准备包含数字的输入
        let input1 = "test123";
        let input2 = "123test";
        let input3 = "test-123-branch";

        // Act: 调用 slugify 方法
        let result1 = BranchNaming::slugify(input1);
        let result2 = BranchNaming::slugify(input2);
        let result3 = BranchNaming::slugify(input3);

        // Assert: 验证数字被保留
        assert_eq!(result1, "test123");
        assert_eq!(result2, "123test");
        assert_eq!(result3, "test-123-branch");
    }

    // ==================== Sanitize Boundary Tests (from naming_advanced.rs) ====================

    /// 测试sanitize方法处理边界情况
    ///
    /// ## 测试目的
    /// 验证 `BranchNaming::sanitize()` 方法能够正确处理各种边界情况（空字符串、空格、连字符、单个字符等）。
    ///
    /// ## 测试场景
    /// 1. 测试空字符串
    /// 2. 测试只有空格的字符串
    /// 3. 测试只有连字符的字符串
    /// 4. 测试单个小写字母
    /// 5. 测试单个大写字母
    ///
    /// ## 预期结果
    /// - 空字符串和空格返回空字符串
    /// - 连字符返回空字符串
    /// - 单个字母返回小写字母
    #[test]
    fn test_sanitize_with_edge_cases_handles_correctly() {
        // Arrange: 准备边界情况输入
        let empty_input = "";
        let whitespace_input = "   ";
        let hyphens_input = "---";
        let single_lowercase = "a";
        let single_uppercase = "A";

        // Act: 调用 sanitize 方法
        let result_empty = BranchNaming::sanitize(empty_input);
        let result_whitespace = BranchNaming::sanitize(whitespace_input);
        let result_hyphens = BranchNaming::sanitize(hyphens_input);
        let result_lowercase = BranchNaming::sanitize(single_lowercase);
        let result_uppercase = BranchNaming::sanitize(single_uppercase);

        // Assert: 验证边界情况处理正确
        assert_eq!(result_empty, "");
        assert_eq!(result_whitespace, "");
        assert_eq!(result_hyphens, "");
        assert_eq!(result_lowercase, "a");
        assert_eq!(result_uppercase, "a");
    }

    /// 测试sanitize方法移除非ASCII字符
    ///
    /// ## 测试目的
    /// 验证 `BranchNaming::sanitize()` 方法能够移除非ASCII字符，保留ASCII字符。
    ///
    /// ## 测试场景
    /// 1. 测试包含Unicode字符的输入（café, naïve, résumé）
    /// 2. 调用sanitize方法
    /// 3. 验证非ASCII字符被移除，ASCII字符被保留
    ///
    /// ## 预期结果
    /// - 非ASCII字符（é, ï等）被移除
    /// - ASCII字符（caf, na, r等）被保留
    #[test]
    fn test_sanitize_with_unicode_characters_removes_non_ascii() {
        // Arrange: 准备包含 Unicode 字符的输入
        let input1 = "café";
        let input2 = "naïve";
        let input3 = "résumé";

        // Act: 调用 sanitize 方法
        let result1 = BranchNaming::sanitize(input1);
        let result2 = BranchNaming::sanitize(input2);
        let result3 = BranchNaming::sanitize(input3);

        // Assert: 验证非 ASCII 字符被移除，ASCII 字符被保留
        assert!(result1.contains("caf"));
        assert!(!result1.contains("é"));
        assert!(result2.contains("na"));
        assert!(!result2.contains("ï"));
        assert!(result3.contains("r"));
        assert!(!result3.contains("é"));
    }

    /// 测试sanitize方法保留数字
    ///
    /// ## 测试目的
    /// 验证 `BranchNaming::sanitize()` 方法能够保留输入中的数字。
    ///
    /// ## 测试场景
    /// 1. 测试包含数字的输入（test123, 123test, test-123-branch）
    /// 2. 调用sanitize方法
    /// 3. 验证数字被保留
    ///
    /// ## 预期结果
    /// - 数字被保留在结果中
    /// - 格式正确（小写、连字符分隔）
    #[test]
    fn test_sanitize_with_numbers_preserves_numbers() {
        // Arrange: 准备包含数字的输入
        let input1 = "test123";
        let input2 = "123test";
        let input3 = "test-123-branch";

        // Act: 调用 sanitize 方法
        let result1 = BranchNaming::sanitize(input1);
        let result2 = BranchNaming::sanitize(input2);
        let result3 = BranchNaming::sanitize(input3);

        // Assert: 验证数字被保留
        assert_eq!(result1, "test123");
        assert_eq!(result2, "123test");
        assert_eq!(result3, "test-123-branch");
    }
}
