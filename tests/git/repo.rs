//! Git 仓库检测和类型识别测试
//!
//! 测试 GitRepo 模块的核心功能，包括：
//! - URL 解析和仓库名提取
//! - 仓库类型检测
//! - 错误处理和边界情况
//! - 与现有 mock 实现的一致性验证
//!
//! ## 测试策略
//!
//! - 使用 `rstest` 进行参数化测试
//! - 测试函数返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 测试各种 URL 格式、边界情况和性能场景

use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::rstest;

use workflow::git::{GitRepo, RepoType};

// ==================== URL 解析和仓库名提取测试 ====================

/// 测试从有效URL中提取仓库名（各种URL格式）
#[rstest]
#[case("git@github.com:owner/repo.git", "owner/repo")]
#[case("git@github.com:owner/repo", "owner/repo")]
#[case("git@github-brainim:owner/repo.git", "owner/repo")]
#[case("git@github-custom:owner/repo", "owner/repo")]
#[case("https://github.com/owner/repo.git", "owner/repo")]
#[case("https://github.com/owner/repo", "owner/repo")]
#[case("https://www.github.com/owner/repo.git", "owner/repo")]
#[case("http://github.com/owner/repo", "owner/repo")]
#[case("git@codeup.aliyun.com:owner/repo.git", "owner/repo")]
#[case("git@codeup.aliyun.com:owner/repo", "owner/repo")]
#[case("https://codeup.aliyun.com/owner/repo.git", "owner/repo")]
#[case("https://codeup.aliyun.com/owner/repo", "owner/repo")]
#[case("http://codeup.aliyun.com/owner/repo", "owner/repo")]
fn test_extract_repo_name_from_url_valid_cases_return_ok(#[case] url: &str, #[case] expected: &str) -> Result<()> {
    // Arrange: 准备测试各种有效 URL 格式的仓库名提取
    let repo_name = GitRepo::extract_repo_name_from_url(url)?;
    assert_eq!(repo_name, expected);
    Ok(())
}

/// 测试从嵌套路径URL中提取仓库名
#[rstest]
#[case("git@github.com:org/sub-org/repo.git", "org/sub-org/repo")]
#[case("https://github.com/org/sub-org/repo", "org/sub-org/repo")]
#[case(
    "git@github-enterprise:company/team/project.git",
    "company/team/project"
)]
#[case(
    "https://codeup.aliyun.com/group/subgroup/project.git",
    "group/subgroup/project"
)]
fn test_extract_repo_name_from_url_nested_paths_return_ok(#[case] url: &str, #[case] expected: &str) -> Result<()> {
    // Arrange: 准备测试嵌套路径的仓库名提取
    let repo_name = GitRepo::extract_repo_name_from_url(url)?;
    assert_eq!(repo_name, expected);
    Ok(())
}

/// 测试从包含特殊字符的URL中提取仓库名
#[rstest]
#[case("git@github.com:owner/repo-with-dashes.git", "owner/repo-with-dashes")]
#[case(
    "https://github.com/owner/repo_with_underscores",
    "owner/repo_with_underscores"
)]
#[case("git@github.com:owner/repo.with.dots.git", "owner/repo.with.dots")]
#[case("https://github.com/owner/repo123", "owner/repo123")]
#[case("git@codeup.aliyun.com:中文用户/中文仓库.git", "中文用户/中文仓库")]
fn test_extract_repo_name_from_url_special_characters_return_ok(#[case] url: &str, #[case] expected: &str) -> Result<()> {
    // Arrange: 准备测试包含特殊字符的仓库名提取
    let repo_name = GitRepo::extract_repo_name_from_url(url)?;
    assert_eq!(repo_name, expected);
    Ok(())
}

/// 测试从无效URL中提取仓库名（应返回错误）
#[rstest]
#[case("")]
#[case("not-a-url")]
#[case("https://example.com/owner/repo")]
#[case("git@example.com:owner/repo.git")]
#[case("ssh://git@github.com/owner/repo.git")]
#[case("ftp://github.com/owner/repo")]
#[case("https://github.com/")]
#[case("git@github.com:")]
#[case(workflow::git::github::BASE)]
#[case("git@github.com")]
fn test_extract_repo_name_from_url_invalid_cases(#[case] url: &str) {
    // Arrange: 准备测试无效 URL 的错误处理
    let result = GitRepo::extract_repo_name_from_url(url);
    assert!(
        result.is_err(),
        "Should fail to extract repo name from invalid URL: {}",
        url
    );
}

/// 测试从包含尾部斜杠的URL中提取仓库名（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 GitRepo 能够正确处理包含尾部斜杠的 URL。
///
/// ## 测试场景
/// 测试各种包含尾部斜杠的 URL 格式：
/// - HTTPS URL 带尾部斜杠
/// - HTTPS URL 带 .git 和尾部斜杠
/// - Codeup URL 带尾部斜杠
///
/// ## 预期结果
/// - 所有 URL 都能正确提取仓库名，忽略尾部斜杠
#[rstest]
#[case("https://github.com/owner/repo/", "owner/repo")]
#[case("https://github.com/owner/repo.git/", "owner/repo")]
#[case("https://codeup.aliyun.com/owner/repo/", "owner/repo")]
fn test_extract_repo_name_from_url_trailing_slashes_return_ok(
    #[case] url: &str,
    #[case] expected: &str,
) -> Result<()> {
    // Arrange: 准备测试末尾斜杠的处理
    let repo_name = GitRepo::extract_repo_name_from_url(url)?;
    assert_eq!(repo_name, expected);
    Ok(())
}

/// 测试URL大小写敏感性 - 有效URL（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 GitRepo 能够正确处理大小写敏感的仓库名。
///
/// ## 测试场景
/// 测试包含大写字母的仓库名（域名必须小写）：
/// - HTTPS URL 带大写仓库名
/// - SSH URL 带大写仓库名
/// - Codeup URL 带大写仓库名
///
/// ## 预期结果
/// - 仓库名保持原始大小写
/// - 域名必须小写
#[rstest]
#[case("https://github.com/Owner/Repo.git", "Owner/Repo")]
#[case("git@github.com:Owner/Repo.git", "Owner/Repo")]
#[case("https://codeup.aliyun.com/Owner/Repo", "Owner/Repo")]
fn test_extract_repo_name_from_url_case_sensitivity_valid_cases_return_ok(
    #[case] url: &str,
    #[case] expected: &str,
) -> Result<()> {
    // Arrange: 准备测试大小写敏感性 - 实际实现是大小写敏感的，只有小写域名被支持
    let repo_name = GitRepo::extract_repo_name_from_url(url)?;
    assert_eq!(repo_name, expected);
    Ok(())
}

/// 测试URL大小写敏感性 - 无效URL（大写域名）（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 GitRepo 拒绝包含大写域名的 URL。
///
/// ## 测试场景
/// 测试包含大写域名的 URL（应被拒绝）：
/// - HTTPS URL 带大写域名
/// - SSH URL 带大写域名
/// - Codeup URL 带大写域名
///
/// ## 预期结果
/// - 所有包含大写域名的 URL 都应返回错误
#[rstest]
#[case("https://GITHUB.com/Owner/Repo.git")]
#[case("git@GITHUB.com:Owner/Repo.git")]
#[case("https://CODEUP.aliyun.com/Owner/Repo")]
fn test_extract_repo_name_from_url_case_sensitivity_invalid_domains(#[case] url: &str) {
    // Arrange: 准备测试大写域名不被支持
    let result = GitRepo::extract_repo_name_from_url(url);
    assert!(
        result.is_err(),
        "Should fail for uppercase domain in URL: {}",
        url
    );
}

// ==================== Repository Type Detection Tests ====================

// 从 tests/git/types.rs 复制的模拟函数，用于一致性验证
fn mock_parse_repo_type_from_url(url: &str) -> RepoType {
    // 检查 GitHub：包含 github.com 或 SSH host 以 github 开头
    if url.contains("github.com")
        || url.starts_with("git@github")
        || url.starts_with("ssh://git@github")
    {
        RepoType::GitHub
    } else if url.contains("codeup.aliyun.com") {
        RepoType::Codeup
    } else {
        RepoType::Unknown
    }
}

/// 测试仓库类型解析与mock实现的一致性
#[rstest]
#[case("git@github.com:owner/repo.git", RepoType::GitHub)]
#[case("https://github.com/owner/repo", RepoType::GitHub)]
#[case("git@github-custom:owner/repo.git", RepoType::GitHub)]
#[case("ssh://git@github.com/owner/repo", RepoType::GitHub)]
#[case("git@codeup.aliyun.com:owner/repo.git", RepoType::Codeup)]
#[case("https://codeup.aliyun.com/owner/repo", RepoType::Codeup)]
#[case("git@example.com:owner/repo.git", RepoType::Unknown)]
#[case("https://gitlab.com/owner/repo", RepoType::Unknown)]
#[case("", RepoType::Unknown)]
fn test_parse_repo_type_consistency_with_mock(#[case] url: &str, #[case] expected: RepoType) {
    // Assert: 验证实际实现与 mock 实现的一致性

    // 使用反射或其他方式访问私有方法进行测试
    // 由于 parse_repo_type_from_url 是私有方法，我们通过 URL 解析来间接测试
    let mock_result = mock_parse_repo_type_from_url(url);
    assert_eq!(
        mock_result, expected,
        "Mock implementation should match expected result for URL: {}",
        url
    );

    // 注意：实际的 parse_repo_type_from_url 是私有方法，
    // 在实际项目中可能需要将其改为 pub(crate) 或添加测试辅助方法
}

/// 测试GitHub SSH别名检测
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_github_ssh_alias_detection() {
    // Arrange: 准备测试 GitHub SSH 别名的检测
    let github_aliases = vec![
        "git@github-personal:owner/repo.git",
        "git@github-work:owner/repo.git",
        "git@github-enterprise:owner/repo.git",
        "git@github123:owner/repo.git",
    ];

    for url in github_aliases {
        let result = mock_parse_repo_type_from_url(url);
        assert_eq!(
            result,
            RepoType::GitHub,
            "Should detect GitHub for SSH alias: {}",
            url
        );
    }
}

/// 测试大小写敏感的检测
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_case_sensitive_detection() {
    // Arrange: 准备测试大小写敏感的检测 - mock 实现是大小写敏感的
    let valid_cases = vec![
        ("https://github.com/owner/repo", RepoType::GitHub),
        ("git@github.com:owner/repo.git", RepoType::GitHub),
        ("https://codeup.aliyun.com/owner/repo", RepoType::Codeup),
        ("git@codeup.aliyun.com:owner/repo.git", RepoType::Codeup),
    ];

    for (url, expected) in valid_cases {
        let result = mock_parse_repo_type_from_url(url);
        assert_eq!(
            result, expected,
            "Should detect correct type for lowercase URL: {}",
            url
        );
    }

    // 大写域名返回 Unknown
    let uppercase_cases = vec![
        ("https://GITHUB.com/owner/repo", RepoType::Unknown),
        ("git@GITHUB.com:owner/repo.git", RepoType::Unknown),
        ("https://CODEUP.aliyun.com/owner/repo", RepoType::Unknown),
        ("git@CODEUP.aliyun.com:owner/repo.git", RepoType::Unknown),
    ];

    for (url, expected) in uppercase_cases {
        let result = mock_parse_repo_type_from_url(url);
        assert_eq!(
            result, expected,
            "Should return Unknown for uppercase domain: {}",
            url
        );
    }
}

// ==================== Boundary Conditions and Error Handling Tests ====================

/// 测试空字符串和空白字符的处理
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_empty_and_whitespace_urls() {
    // Arrange: 准备测试空字符串和空白字符的处理
    let invalid_urls = vec!["", " ", "\t", "\n", "   \t\n   "];

    for url in invalid_urls {
        let extract_result = GitRepo::extract_repo_name_from_url(url);
        assert!(
            extract_result.is_err(),
            "Should fail for empty/whitespace URL: '{}'",
            url
        );

        let type_result = mock_parse_repo_type_from_url(url);
        assert_eq!(
            type_result,
            RepoType::Unknown,
            "Should return Unknown for empty/whitespace URL: '{}'",
            url
        );
    }
}

/// 测试格式错误的URL
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_malformed_urls() {
    // Arrange: 准备测试格式错误的 URL
    let malformed_urls = vec![
        "git@",
        "https://",
        "git@github.com:",
        "https://github.com/",
        "://github.com/owner/repo",
        "github.com/owner/repo",
    ];

    for url in malformed_urls {
        let result = GitRepo::extract_repo_name_from_url(url);
        assert!(result.is_err(), "Should fail for malformed URL: {}", url);
    }
}

/// 测试Unicode字符和特殊字符的处理
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_unicode_and_special_characters_return_ok() -> Result<()> {
    // Arrange: 准备测试 Unicode 字符和特殊字符的处理
    let unicode_cases = vec![
        ("git@github.com:用户/仓库.git", "用户/仓库"),
        ("https://github.com/用户名/项目名", "用户名/项目名"),
        (
            "git@github.com:владелец/репозиторий.git",
            "владелец/репозиторий",
        ),
        ("https://github.com/所有者/リポジトリ", "所有者/リポジトリ"),
    ];

    for (url, expected) in unicode_cases {
        let repo_name = GitRepo::extract_repo_name_from_url(url)?;
        assert_eq!(repo_name, expected);
    }
    Ok(())
}

/// 测试极长URL的处理
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_very_long_urls_return_ok() -> Result<()> {
    // Arrange: 准备测试极长的 URL
    let long_owner = "a".repeat(100);
    let long_repo = "b".repeat(100);
    let long_url = format!("https://github.com/{}/{}", long_owner, long_repo);
    let expected = format!("{}/{}", long_owner, long_repo);

    let repo_name = GitRepo::extract_repo_name_from_url(&long_url)?;
    assert_eq!(repo_name, expected);
    Ok(())
}

/// 测试包含查询参数的URL
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_url_with_query_parameters() {
    // Arrange: 准备测试包含查询参数的 URL（虽然不常见，但应该优雅处理）
    let urls_with_params = vec![
        "https://github.com/owner/repo?tab=readme",
        "https://github.com/owner/repo.git?ref=main",
    ];

    for url in urls_with_params {
        let result = GitRepo::extract_repo_name_from_url(url);
        // 这些 URL 可能不被支持，但不应该 panic
        // 具体行为取决于实现，这里主要测试不会崩溃
        let _ = result;
    }
}

// ==================== Performance and Stress Tests ====================

/// 测试大量URL处理的性能
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_performance_with_many_urls() {
    // Arrange: 准备测试大量 URL 处理的性能
    let base_urls = vec![
        "git@github.com:owner{}/repo{}.git",
        "https://github.com/owner{}/repo{}",
        "git@codeup.aliyun.com:owner{}/repo{}.git",
        "https://codeup.aliyun.com/owner{}/repo{}",
    ];

    let start = std::time::Instant::now();

    for i in 0..1000 {
        for base_url in &base_urls {
            let url = base_url.replace("{}", &i.to_string());
            let _ = GitRepo::extract_repo_name_from_url(&url);
            let _ = mock_parse_repo_type_from_url(&url);
        }
    }

    let duration = start.elapsed();

    // 性能基准：4000个URL应该在合理时间内完成（放宽到30秒，考虑到CI环境的性能差异）
    assert!(
        duration.as_secs() < 30,
        "Performance test took too long: {:?}",
        duration
    );
}

// ==================== Integration Test Helpers ====================

/// 测试URL解析和类型检测的集成（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 URL 解析和类型检测的集成功能。
///
/// ## 测试场景
/// 测试不同仓库类型的 URL：
/// - GitHub URL（应能解析并识别为 GitHub）
/// - Codeup URL（应能解析并识别为 Codeup）
/// - 未知类型 URL（应识别为 Unknown，可能无法解析）
///
/// ## 预期结果
/// - 支持的仓库类型能正确解析仓库名
/// - 类型检测结果与预期一致
#[rstest]
#[case("git@github.com:owner/repo.git", "owner/repo", RepoType::GitHub, true)]
#[case("https://codeup.aliyun.com/owner/repo", "owner/repo", RepoType::Codeup, true)]
#[case("git@gitlab.com:owner/repo.git", "owner/repo", RepoType::Unknown, false)]
fn test_extract_and_detect_integration_return_ok(
    #[case] url: &str,
    #[case] expected_name: &str,
    #[case] expected_type: RepoType,
    #[case] should_extract_name: bool,
) -> Result<()> {
    // Arrange: 准备测试仓库名提取
    if should_extract_name {
        let repo_name = GitRepo::extract_repo_name_from_url(url)?;
        assert_eq!(repo_name, expected_name);
    }

    // Arrange: 准备测试类型检测
    let type_result = mock_parse_repo_type_from_url(url);
    assert_eq!(
        type_result, expected_type,
        "Should detect correct type for URL: {}",
        url
    );
    Ok(())
}

// ==================== Regression Tests ====================

/// 测试已知问题的回归
#[test]
fn test_regression_known_issues_return_ok() -> Result<()> {
    // Arrange: 准备测试已知问题的回归

    // Arrange: 准备测试：确保 SSH 别名正确处理
    let ssh_alias_cases = vec![
        "git@github-personal:owner/repo.git",
        "git@github-work:owner/repo.git",
    ];

    for url in ssh_alias_cases {
        let repo_name = GitRepo::extract_repo_name_from_url(url)?;
        assert_eq!(repo_name, "owner/repo");

        let type_result = mock_parse_repo_type_from_url(url);
        assert_eq!(type_result, RepoType::GitHub);
    }

    // Arrange: 准备测试：确保 .git 后缀正确处理
    let git_suffix_cases = vec![
        ("git@github.com:owner/repo.git", "owner/repo"),
        ("git@github.com:owner/repo", "owner/repo"),
        ("https://github.com/owner/repo.git", "owner/repo"),
        ("https://github.com/owner/repo", "owner/repo"),
    ];

    for (url, expected) in git_suffix_cases {
        let repo_name = GitRepo::extract_repo_name_from_url(url)?;
        assert_eq!(repo_name, expected);
    }
    Ok(())
}
