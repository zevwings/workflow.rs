//! Jira 辅助函数
//!
//! 本模块提供了 Jira 相关的辅助函数，包括：
//! - 字符串处理（提取项目名、提取 ticket ID、验证格式）
//! - 文件名处理（邮箱地址清理）
//! - 认证和 URL 构建（获取认证信息、构建基础 URL）
//!
//! 注意：日志处理相关的辅助函数已迁移到 `jira::logs::helpers` 模块。

use crate::base::constants::errors::validation_errors;
use crate::base::settings::Settings;
use color_eyre::Result;
use regex::Regex;

/// 从 Jira ticket 提取项目名
///
/// # 示例
/// ```
/// use workflow::jira::helpers::extract_jira_project;
/// assert_eq!(extract_jira_project("PROJ-123"), Some("PROJ"));
/// assert_eq!(extract_jira_project("PROJ"), None);
/// ```
pub fn extract_jira_project(ticket: &str) -> Option<&str> {
    ticket.split('-').next().filter(|s| *s != ticket)
}

/// 验证 Jira ticket 格式
///
/// Jira ticket 应该是 PROJECT-123 格式（ticket），或纯项目名（PROJECT）。
/// 项目名只能包含字母、数字和下划线。
///
/// # 示例
/// ```
/// use workflow::jira::helpers::validate_jira_ticket_format;
/// assert!(validate_jira_ticket_format("PROJ-123").is_ok());
/// assert!(validate_jira_ticket_format("PROJ").is_ok());
/// assert!(validate_jira_ticket_format("PROJ-123-456").is_ok());
/// assert!(validate_jira_ticket_format("invalid/ticket").is_err());
/// ```
pub fn validate_jira_ticket_format(ticket: &str) -> Result<()> {
    // 先检查是否为空或只包含空白字符
    if ticket.trim().is_empty() {
        color_eyre::eyre::bail!(
            "{}: {}",
            validation_errors::INVALID_JIRA_ID_FORMAT,
            validation_errors::JIRA_ID_EMPTY
        );
    }

    let is_valid_format: bool = if let Some(project) = extract_jira_project(ticket) {
        // 如果是 ticket 格式（PROJ-123），需要：
        // 1. 项目名有效（只包含字母、数字、下划线）
        // 2. ticket 必须包含数字部分（PROJ-123 格式，不能只是 PROJ-）
        let project_valid = project.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');

        // 检查是否有数字部分（ticket 格式应该是 PROJECT-NUMBER）
        let has_number_part =
            ticket.split('-').skip(1).any(|part| part.chars().any(|c| c.is_ascii_digit()));

        project_valid && has_number_part
    } else {
        // 如果是项目名格式，检查是否只包含有效字符
        ticket.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    };

    if !is_valid_format {
        color_eyre::eyre::bail!(
            "{}: '{}'. {}\n  - Ticket names should contain only letters, numbers, and hyphens\n  - Project names should contain only letters, numbers, and underscores\n  - Do not use branch names or paths (e.g., 'zw/修改打包脚本问题')",
            validation_errors::INVALID_JIRA_ID_FORMAT,
            ticket,
            validation_errors::JIRA_ID_FORMAT_HELP
        );
    }

    Ok(())
}

/// 从 PR 标题提取 Jira ticket ID
///
/// # 示例
/// ```
/// use workflow::jira::helpers::extract_jira_ticket_id;
/// assert_eq!(extract_jira_ticket_id("PROJ-123: Fix bug"), Some("PROJ-123".to_string()));
/// assert_eq!(extract_jira_ticket_id("Fix bug"), None);
/// ```
pub fn extract_jira_ticket_id(pull_request_title: &str) -> Option<String> {
    // 匹配格式: PROJ-123 或 PROJ-123:
    let re = Regex::new(r"^([A-Z]+-\d+)").ok()?;
    re.captures(pull_request_title)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// 清理邮箱地址作为文件名（方案1：简单替换）
///
/// 将邮箱地址中的特殊字符替换为安全的文件名字符：
/// - `@` → `_at_`
/// - `.` → `_dot_`
/// - `+` → `_plus_`
///
/// # 示例
/// ```
/// use workflow::jira::helpers::sanitize_email_for_filename;
/// assert_eq!(sanitize_email_for_filename("user@example.com"), "user_at_example_dot_com");
/// assert_eq!(sanitize_email_for_filename("user+tag@example.com"), "user_plus_tag_at_example_dot_com");
/// ```
pub fn sanitize_email_for_filename(email: &str) -> String {
    email.replace('@', "_at_").replace('.', "_dot_").replace('+', "_plus_")
}

/// 获取认证信息
///
/// 从配置文件中读取 Jira API 认证所需的 email 和 api_token。
///
/// # 返回
///
/// 返回 `(email, api_token)` 元组。
pub fn get_auth() -> Result<(String, String)> {
    let settings = Settings::get();
    let email = settings.jira.email.clone().unwrap_or_default();
    let api_token = settings.jira.api_token.clone().unwrap_or_default();
    Ok((email, api_token))
}

/// 获取 Jira API 基础 URL
///
/// 从配置文件中读取 Jira 服务地址，并构建 REST API 基础 URL。
/// 格式：`{jira_service_address}/rest/api/2`
///
/// # 返回
///
/// 返回完整的 REST API 基础 URL。
///
/// # 错误
///
/// 如果 `jira_service_address` 未设置或为空，返回错误。
pub fn get_base_url() -> Result<String> {
    let settings = Settings::get();
    let base_url = settings.jira.service_address.clone().unwrap_or_default();

    if base_url.is_empty() {
        color_eyre::eyre::bail!(
            "Jira service address is not configured. \
            Please run 'workflow setup' to configure it."
        );
    }

    Ok(format!("{}/rest/api/2", base_url))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // ==================== Jira Project Extraction Tests ====================

    /// 测试从Jira ticket中提取项目名
    ///
    /// ## 测试目的
    /// 验证 `extract_jira_project()` 函数能够从有效的Jira ticket ID中正确提取项目名。
    ///
    /// ## 测试场景
    /// 1. 测试有效的ticket ID（PROJ-123, PROJ-456, TEST-789）
    /// 2. 测试无效输入（只有项目名、空字符串）
    ///
    /// ## 预期结果
    /// - 有效ticket ID返回项目名（PROJ, TEST等）
    /// - 无效输入返回None
    #[test]
    fn test_extract_jira_project_with_valid_ticket_returns_project() {
        // Arrange: 准备有效的 Jira ticket ID
        let tickets = ["PROJ-123", "PROJ-456", "TEST-789"];

        // Act & Assert: 验证提取项目名正确
        assert_eq!(extract_jira_project(tickets[0]), Some("PROJ"));
        assert_eq!(extract_jira_project(tickets[1]), Some("PROJ"));
        assert_eq!(extract_jira_project(tickets[2]), Some("TEST"));
        assert_eq!(extract_jira_project("PROJ"), None);
        assert_eq!(extract_jira_project(""), None);
    }

    // ==================== Jira Ticket ID Extraction Tests ====================

    /// 测试从字符串中提取Jira ticket ID
    ///
    /// ## 测试目的
    /// 验证 `extract_jira_ticket_id()` 函数能够从包含ticket ID的字符串中正确提取ticket ID。
    ///
    /// ## 测试场景
    /// 1. 测试包含ticket ID的字符串（"PROJ-123: Fix bug", "PROJ-123"等）
    /// 2. 测试不包含ticket ID的字符串（"Fix bug", ""）
    ///
    /// ## 预期结果
    /// - 包含ticket ID的字符串返回ticket ID（"PROJ-123", "PROJ-456"等）
    /// - 不包含ticket ID的字符串返回None
    #[test]
    fn test_extract_jira_ticket_id_with_valid_strings_returns_ticket_id() {
        // Arrange: 准备包含 ticket ID 的字符串
        let inputs = ["PROJ-123: Fix bug", "PROJ-123", "PROJ-456: Add feature"];

        // Act & Assert: 验证提取 ticket ID 正确
        assert_eq!(
            extract_jira_ticket_id(inputs[0]),
            Some("PROJ-123".to_string())
        );
        assert_eq!(
            extract_jira_ticket_id(inputs[1]),
            Some("PROJ-123".to_string())
        );
        assert_eq!(
            extract_jira_ticket_id(inputs[2]),
            Some("PROJ-456".to_string())
        );
        assert_eq!(extract_jira_ticket_id("Fix bug"), None);
        assert_eq!(extract_jira_ticket_id(""), None);
    }

    // ==================== Email Sanitization Tests ====================

    /// 测试清理邮箱地址用于文件名
    ///
    /// ## 测试目的
    /// 验证 `sanitize_email_for_filename()` 函数能够将邮箱地址转换为适合文件名的格式（替换特殊字符）。
    ///
    /// ## 测试场景
    /// 1. 测试各种格式的邮箱地址（普通邮箱、带+号的邮箱、多级域名等）
    /// 2. 测试空字符串
    ///
    /// ## 预期结果
    /// - 邮箱地址中的特殊字符被替换（@ -> _at_, . -> _dot_, + -> _plus_）
    /// - 空字符串返回空字符串
    #[test]
    fn test_sanitize_email_for_filename_with_various_emails_returns_sanitized() {
        // Arrange: 准备各种格式的邮箱地址
        let emails = [
            ("user@example.com", "user_at_example_dot_com"),
            ("user+tag@example.com", "user_plus_tag_at_example_dot_com"),
            (
                "test.user@example.co.uk",
                "test_dot_user_at_example_dot_co_dot_uk",
            ),
            ("", ""),
        ];

        // Act & Assert: 验证邮箱地址被正确清理
        for (email, expected) in emails.iter() {
            assert_eq!(sanitize_email_for_filename(email), *expected);
        }
    }

    // ==================== Jira Ticket Format Validation Tests ====================

    /// 测试验证Jira ticket格式（有效格式）
    ///
    /// ## 测试目的
    /// 验证 `validate_jira_ticket_format()` 函数能够正确识别有效的Jira ticket格式。
    ///
    /// ## 测试场景
    /// 1. 测试各种有效格式（PROJ-123, PROJ, TEST-456, PROJ-123-456, PROJECT_123）
    ///
    /// ## 预期结果
    /// - 所有有效格式通过验证，返回Ok
    #[test]
    fn test_validate_jira_ticket_format_with_valid_formats_returns_ok() {
        // Arrange: 准备有效的 ticket 格式
        let valid_tickets = [
            "PROJ-123",
            "PROJ",
            "TEST-456",
            "PROJ-123-456",
            "PROJECT_123",
        ];

        // Act & Assert: 验证所有有效格式通过验证
        for ticket in valid_tickets.iter() {
            assert!(
                validate_jira_ticket_format(ticket).is_ok(),
                "Ticket '{}' should be valid",
                ticket
            );
        }
    }

    /// 测试验证Jira ticket格式（无效格式）
    ///
    /// ## 测试目的
    /// 验证 `validate_jira_ticket_format()` 函数能够正确识别无效的Jira ticket格式并返回错误。
    ///
    /// ## 测试场景
    /// 1. 测试各种无效格式（空字符串、只有空格、PROJ-、包含斜杠、包含字母等）
    ///
    /// ## 预期结果
    /// - 所有无效格式返回错误
    #[test]
    fn test_validate_jira_ticket_format_with_invalid_formats_returns_error() {
        // Arrange: 准备无效的 ticket 格式
        let invalid_tickets = ["", "   ", "PROJ-", "invalid/ticket", "PROJ-abc"];

        // Act & Assert: 验证所有无效格式返回错误
        for ticket in invalid_tickets.iter() {
            assert!(
                validate_jira_ticket_format(ticket).is_err(),
                "Ticket '{}' should be invalid",
                ticket
            );
        }
    }

    /// 测试验证Jira ticket格式（边界情况）
    ///
    /// ## 测试目的
    /// 验证 `validate_jira_ticket_format()` 函数能够正确处理边界情况的ticket格式。
    ///
    /// ## 测试场景
    /// 1. 测试边界情况（最短格式A-1、长数字PROJECT-999999、下划线格式PROJ_123、多段格式PROJ-123-456-789）
    ///
    /// ## 预期结果
    /// - 所有边界情况通过验证，返回Ok
    #[test]
    fn test_validate_jira_ticket_format_edge_cases_with_edge_cases_returns_ok() {
        // Arrange: 准备边界情况的 ticket 格式
        let edge_cases = ["A-1", "PROJECT-999999", "PROJ_123", "PROJ-123-456-789"];

        // Act & Assert: 验证边界情况通过验证
        for ticket in edge_cases.iter() {
            assert!(
                validate_jira_ticket_format(ticket).is_ok(),
                "Edge case ticket '{}' should be valid",
                ticket
            );
        }
    }
}
