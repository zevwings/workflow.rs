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
use anyhow::Result;

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
                        if use_prefix_format {
                            // Template already includes prefix, just return
                            Ok(rendered)
                        } else {
                            // Convert prefix/ticket-slug to ticket--slug format
                            Ok(Self::convert_to_double_dash_format(&rendered, ticket_id))
                        }
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
            return Ok(format!("{}/{}", branch_type, branch_name_slug));
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

        Ok(rendered)
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

        // If repository-level branch_prefix exists, add prefix
        if let Some(prefix) = RepoConfig::get_branch_prefix() {
            let trimmed = prefix.trim();
            if !trimmed.is_empty() {
                branch_name = format!("{}/{}", trimmed, branch_name);
            }
        }

        Ok(branch_name)
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
                Ok(Self::format_branch_name_with_ticket(
                    use_prefix_format,
                    ticket_id,
                    &base_name,
                ))
            }
            Err(e) => {
                log_warning!(
                    "Failed to generate branch name using LLM: {}, falling back to simple method",
                    e
                );
                // Fallback to simple method
                let slug = Self::slugify(summary);
                Ok(Self::format_branch_name_simple(
                    use_prefix_format,
                    ticket_id,
                    &slug,
                ))
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

        if sanitized.is_empty() {
            // If sanitization removed everything, try slugify
            let slug = Self::slugify(&text_to_sanitize);
            if slug.is_empty() {
                anyhow::bail!(
                    "Branch name cannot be empty after sanitization. Please use English characters or provide a JIRA ticket ID."
                );
            }
            Ok(slug)
        } else {
            Ok(sanitized)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== sanitize 函数测试 ====================

    #[test]
    fn test_sanitize_basic() {
        assert_eq!(BranchNaming::sanitize("Hello World"), "hello-world");
        assert_eq!(BranchNaming::sanitize("test-branch"), "test-branch");
        assert_eq!(BranchNaming::sanitize("Test Branch"), "test-branch");
    }

    #[test]
    fn test_sanitize_special_characters() {
        assert_eq!(BranchNaming::sanitize("test@branch#123"), "test-branch-123");
        assert_eq!(BranchNaming::sanitize("test.branch"), "test-branch");
        assert_eq!(BranchNaming::sanitize("test_branch"), "test-branch");
    }

    #[test]
    fn test_sanitize_non_ascii() {
        // 非 ASCII 字符应该被忽略
        assert_eq!(BranchNaming::sanitize("测试分支"), "");
        // 中文被忽略，但 test 和 branch 之间没有分隔符，所以不会添加连字符
        assert_eq!(BranchNaming::sanitize("test中文branch"), "testbranch");
        // 中间有空格，所以会添加连字符
        assert_eq!(BranchNaming::sanitize("test 中文 branch"), "test-branch");
        assert_eq!(BranchNaming::sanitize("Hello 世界"), "hello");
    }

    #[test]
    fn test_sanitize_duplicate_hyphens() {
        assert_eq!(BranchNaming::sanitize("test---branch"), "test-branch");
        assert_eq!(BranchNaming::sanitize("test   branch"), "test-branch");
    }

    #[test]
    fn test_sanitize_trim_dashes() {
        assert_eq!(BranchNaming::sanitize("-test-branch-"), "test-branch");
        assert_eq!(BranchNaming::sanitize("--test--"), "test");
    }

    // ==================== slugify 函数测试 ====================

    #[test]
    fn test_slugify_basic() {
        assert_eq!(BranchNaming::slugify("Hello World"), "hello-world");
        assert_eq!(BranchNaming::slugify("test branch"), "test-branch");
    }

    #[test]
    fn test_slugify_preserves_underscores() {
        assert_eq!(BranchNaming::slugify("test_branch"), "test_branch");
        assert_eq!(
            BranchNaming::slugify("test_branch_name"),
            "test_branch_name"
        );
    }

    #[test]
    fn test_slugify_length_limit() {
        let long_text = "a".repeat(100);
        let result = BranchNaming::slugify(&long_text);
        assert!(result.len() <= 50);
    }

    #[test]
    fn test_slugify_filters_empty_segments() {
        assert_eq!(BranchNaming::slugify("test---branch"), "test-branch");
        assert_eq!(BranchNaming::slugify("test   branch"), "test-branch");
    }

    // ==================== from_type_and_slug 函数测试 ====================

    #[test]
    fn test_from_type_and_slug_without_ticket() {
        let result = BranchNaming::from_type_and_slug("feature", "my-branch", None).unwrap();
        assert_eq!(result, "feature/my-branch");
    }

    #[test]
    fn test_from_type_and_slug_with_ticket() {
        // 注意：这个测试需要模板系统，可能会失败，但至少测试了函数调用
        let result = BranchNaming::from_type_and_slug("feature", "my-branch", Some("PROJ-123"));
        // 结果取决于模板系统，但应该包含类型和 slug
        assert!(result.is_ok());
        let branch_name = result.unwrap();
        assert!(branch_name.contains("feature") || branch_name.contains("my-branch"));
    }
}
